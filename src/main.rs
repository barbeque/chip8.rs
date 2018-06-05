extern crate sdl2;
extern crate rand;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::path::PathBuf;
use std::time::Duration;
use std::fs::File;
use std::io::prelude::*;

mod opcodes;
use opcodes::*;

struct ComputerState {
    // 4K main memory
    memory: [u8; 4096],
    // General purpose registers V0..VE + special, VF
    registers: [u8; 16],
    // Index register
    index: u16,
    // Program counter
    program_counter: Chip8Address,
    // 2K Video memory
    gfx: [u8; 64 * 32],
    // Delay timer
    delay_timer: u8,
    // Sound timer
    sound_timer: u8,
    // Stack
    stack: [u16; 16],
    // Stack pointer
    sp: u16,
    // Keyboard state
    keys: [u8; 16],
}

impl ComputerState {
    pub fn new() -> ComputerState {
        ComputerState {
            memory: [0u8; 4096],
            registers: [0u8; 16],
            index: 0,
            program_counter: 0x200, // the start of program memory
            gfx: [0u8; (64 * 32)],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0u16; 16],
            sp: 0,
            keys: [0u8; 16],
        }

        // TODO: set up a panic handler that lets us know which IP is illegal
        // TODO: load ROM contents (font set)
    }

    fn advance_pc(&mut self) {
        // advance the instruction pointer
        self.program_counter += 2; // 2 bytes (16 bit instructions)
    }

    pub fn load_program(&mut self, path: &str) {
        let relative_path = PathBuf::from(path);
        let mut absolute_path = std::env::current_dir().unwrap();
        absolute_path.push(relative_path);
        let target = absolute_path.as_path();

        if !target.exists() {
            // Not found...
            let target_path = target.to_str().unwrap(); // FIXME: might cause a crash if it's a really mangled string. need a safer unwrap.
            panic!(
                "Could not find the file '{}' to load as a CHIP-8 program.",
                target_path
            );
        }

        // Load the program into RAM
        let target_path = target.to_str().unwrap();
        println!("Loading CHIP-8 program '{}'", target_path);

        let mut file = File::open(target_path).unwrap();
        let mut buf = Vec::<u8>::new();

        match file.read_to_end(&mut buf) {
            Ok(length) => {
                println!("Loaded {} byte(s)", length);

                if length > self.memory.len() - 0x200 {
                    println!("Warning: too big for main memory. Will probably crash.");
                }

                // Start loading at 0x200
                for (i, b) in buf.iter().enumerate() {
                    self.memory[i + 0x200] = *b;
                }
            },
            Err(e) => {
                println!("Error loading file: {}", e);
                std::process::exit(1);
            }
        }
    }

    pub fn decode(&self, instruction: u16) -> Chip8Opcode {
        // Instructions are stored big-endian so we're good

        // FIXME: This is way too long - can we write a macro that uses like the "0XYZ" notation?
        let top_nibble = (instruction & 0xf000) >> 12;
        if top_nibble == 0x0 {
            if instruction == 0x00e0 {
                return Chip8Opcode::DisplayClear;
            }
            else if instruction == 0x00ee {
                return Chip8Opcode::ReturnFromSubroutine;
            }
            else {
                // 0NNN - call
                return Chip8Opcode::Call(instruction & 0xfff);
            }
        }
        else if top_nibble == 0x1 {
            // 1NNN - jump
            return Chip8Opcode::Goto(instruction & 0xfff);
        }
        else if top_nibble == 0x2 {
            // 2NNN - call sub at NNN
            return Chip8Opcode::CallSub(instruction & 0xfff);
        }
        else if top_nibble == 0x3 {
            // 3xnn - skip next if Vx equal NN
            let register = ((instruction & 0x0f00) >> 8) as u8;
            let data = (instruction & 0x00ff) as u8;
            return Chip8Opcode::SkipNextIfEqual(register, data);
        }
        else if top_nibble == 0x4 {
            // 4xnn - skip next if Vx not equal to NN
            let register = ((instruction & 0x0f00) >> 8) as u8;
            let data = (instruction & 0x00ff) as u8;
            return Chip8Opcode::SkipNextIfNotEqual(register, data);
        }
        else if top_nibble == 0x5 {
            // 5xy0 - skip next if Vx = Vy
            if instruction & 0x000f == 0 {
                let x_register = ((instruction & 0x0f00) >> 8) as u8;
                let y_register = ((instruction & 0x00f0) >> 4) as u8;
                return Chip8Opcode::SkipNextIfRegistersEqual(x_register, y_register);
            } else {
                panic!("Malformed skip next if register equal instruction {:x}", instruction);
            }
        }
        else if top_nibble == 0x6 {
            // assign
            let register = ((instruction & 0x0f00) >> 8) as u8;
            let value = (instruction & 0x00ff) as u8;
            return Chip8Opcode::SetRegister(register, value);
        }
        else if top_nibble == 0x7 {
            // increment w/o carry
            let register = ((instruction & 0x0f00) >> 8) as u8;
            let value = (instruction & 0x00ff) as u8;
            return Chip8Opcode::IncrementRegister(register, value);
        }
        else if top_nibble == 0x8 {
            let x_register = ((instruction & 0x0f00) >> 8) as u8;
            let y_register = ((instruction & 0x00f0) >> 4) as u8;

            let mode = instruction & 0x000f;

            if mode == 0 {
                // 8xy0 - set register to register
                return Chip8Opcode::SetRegisterToRegister(x_register, y_register);
            }
            else if mode == 1 {
                return Chip8Opcode::RegisterRegisterOr(x_register, y_register);
            }
            else if mode == 2 {
                return Chip8Opcode::RegisterRegisterAnd(x_register, y_register);
            }
            else if mode == 3 {
                return Chip8Opcode::RegisterRegisterXor(x_register, y_register);
            }
            else if mode == 4 {
                return Chip8Opcode::IncrementRegisterWithRegister(x_register, y_register);
            }
            else if mode == 5 {
                return Chip8Opcode::DecrementRegisterWithRegister(x_register, y_register);
            }
            else if mode == 6 {
                return Chip8Opcode::ShiftRegisterByRegister(x_register, y_register);
            }
            else if mode == 7 {
                // y minus x - remember, still stored in x, y order
                return Chip8Opcode::YRegisterMinusXRegister(x_register, y_register);
            }
            else if mode == 0xe {
                return Chip8Opcode::LeftShiftRegisterByRegister(x_register, y_register);
            }
            else {
                panic!("Malformed set register to register instruction {:x}", instruction);
            }
        }
        else if top_nibble == 0x9 {
            // skip next if Vx != Vy
            if instruction & 0x000f == 0 {
                let x_register = ((instruction & 0x0f00) >> 8) as u8;
                let y_register = ((instruction & 0x00f0) >> 4) as u8;
                return Chip8Opcode::SkipNextIfRegistersNotEqual(x_register, y_register);
            } else {
                panic!("Malformed skip next if register not-equal instruction {:x}", instruction);
            }
        }
        else if top_nibble == 0xa {
            // set index
            return Chip8Opcode::SetIndexRegister(instruction & 0xfff);
        }
        else if top_nibble == 0xb {
            // far jump
            return Chip8Opcode::JumpFromV0(instruction & 0xfff);
        }
        else if top_nibble == 0xc {
            // random
            let register = ((instruction & 0x0f00) >> 8) as Chip8Register;
            let and_this = (instruction & 0x00ff) as u8;
            return Chip8Opcode::Random(register, and_this);
        }
        else if top_nibble == 0xd {
            // draw sprite
            // DXYN
            let x_register = ((instruction & 0x0f00) >> 8) as Chip8Register;
            let y_register = ((instruction & 0x00f0) >> 4) as Chip8Register;
            let sprite = (instruction & 0x000f) as u8;
            return Chip8Opcode::Draw(x_register, y_register, sprite)
        }
        else if top_nibble == 0xe {
            // key operations depending on bottom byte
            let bottom_byte = instruction & 0xff;
            let register = ((instruction & 0x0f00) >> 8) as u8;
            if bottom_byte == 0x9e {
                // skip if key stored in Vx is pressed
                return Chip8Opcode::SkipNextIfKeyDown(register);
            }
            else if bottom_byte == 0xa1 {
                // skip if key stored in Vx is not pressed
                return Chip8Opcode::SkipNextIfKeyUp(register);
            }
            else {
                panic!("Malformed key press instruction {:x}", instruction);
            }
        }
        else if top_nibble == 0xf {
            let bottom_byte = instruction & 0xff;
            let register = ((instruction & 0x0f00) >> 8) as Chip8Register;
            if bottom_byte == 0x07 { // fx07
                return Chip8Opcode::ReadDelayTimer(register);
            }
            else if bottom_byte == 0x0a { // fx0a
                return Chip8Opcode::BlockOnKeyPress(register);
            }
            else if bottom_byte == 0x15 {
                return Chip8Opcode::SetDelayTimer(register);
            }
            else if bottom_byte == 0x18 {
                return Chip8Opcode::SetSoundTimer(register);
            }
            else if bottom_byte == 0x1e {
                return Chip8Opcode::AddToIndexRegister(register);
            }
            else if bottom_byte == 0x29 {
                return Chip8Opcode::UseSprite(register);
            }
            else if bottom_byte == 0x33 {
                return Chip8Opcode::ReadRegisterAsBCD(register);
            }
            else if bottom_byte == 0x55 {
                return Chip8Opcode::DumpRegisters(register);
            }
            else if bottom_byte == 0x65 { // fx65
                return Chip8Opcode::FillRegisters(register);
            }
            else {
                panic!("Malformed extended instruction {:x}", instruction);
            }
        }

        panic!("Don't know how to decode {} yet.", instruction);
    }

    pub fn execute(&mut self, op: Chip8Opcode) {
        match op {
            // TODO: Call
            Chip8Opcode::DisplayClear => {
                for i in 0..self.gfx.len() {
                    // FIXME: is a more succinct way to do this?
                    self.gfx[i] = 0;
                }
            },
            // TODO: Return from sub
            Chip8Opcode::Goto(address) => {
                self.program_counter = address;
            },
            Chip8Opcode::SkipNextIfEqual(r1, value) => {
                let v1 = self.get_register(r1);
                if v1 == value {
                    // jump ahead one instruction,
                    // fetch will jump automatically
                    self.advance_pc();
                }
            }
            Chip8Opcode::SkipNextIfNotEqual(r1, value) => {
                let v1 = self.get_register(r1);
                if v1 != value {
                    self.advance_pc();
                }
            },
            Chip8Opcode::SkipNextIfRegistersEqual(r1, r2) => {
                let v1 = self.get_register(r1);
                let v2 = self.get_register(r2);
                if v1 == v2 {
                    self.advance_pc();
                }
            },
            Chip8Opcode::SetRegister(r1, value) => {
                self.set_register(r1, value);
            },
            Chip8Opcode::IncrementRegister(r1, step) => {
                let value = self.get_register(r1);
                self.set_register(r1, value.wrapping_add(step));
            },
            Chip8Opcode::SetRegisterToRegister(r1, r2) => {
                let new_value = self.get_register(r2);
                self.set_register(r1, new_value);
            },
            Chip8Opcode::RegisterRegisterOr(r1, r2) => {
                let v1 = self.get_register(r1);
                let v2 = self.get_register(r2);
                self.set_register(r1, v1 | v2);
            },
            Chip8Opcode::RegisterRegisterAnd(r1, r2) => {
                let v1 = self.get_register(r1);
                let v2 = self.get_register(r2);
                self.set_register(r1, v1 & v2);
            },
            Chip8Opcode::RegisterRegisterXor(r1, r2) => {
                let v1 = self.get_register(r1);
                let v2 = self.get_register(r2);
                self.set_register(r1, v1 ^ v2);
            },
            Chip8Opcode::IncrementRegisterWithRegister(r1, r2) => {
                let value = self.get_register(r1);
                let step = self.get_register(r2);

                self.set_register(r1, value.wrapping_add(step));

                if (value as u16 + step as u16) > 255 {
                    // carry
                    self.set_register(0xf, 1);
                }
                else {
                    self.set_register(0xf, 0);
                }
            },
            Chip8Opcode::DecrementRegisterWithRegister(r1, r2) => {
                let value = self.get_register(r1);
                let step = self.get_register(r2);
                self.set_register(r1, value.wrapping_sub(step));

                if value > step {
                    // NOT borrow
                    self.set_register(0xf, 1);
                }
                else {
                    self.set_register(0xf, 0);
                }
            },
            Chip8Opcode::ShiftRegisterByRegister(r1, r2) => {
                let v2 = self.get_register(r2);
                let lsb = v2 & 0x01;
                let value = v2 >> 1;

                self.set_register(r1, value);

                // Set VF to the LSb of v2 before shift
                self.set_register(0xf, lsb);
            },
            Chip8Opcode::YRegisterMinusXRegister(x, y) => {
                let v1 = self.get_register(x);
                let v2 = self.get_register(y);

                self.set_register(x, v2.wrapping_sub(v1));

                if v2 > v1 {
                    // NOT borrow
                    self.set_register(0xf, 1);
                }
                else {
                    self.set_register(0xf, 0);
                }
            },
            Chip8Opcode::SkipNextIfRegistersNotEqual(r1, r2) => {
                let v1 = self.get_register(r1);
                let v2 = self.get_register(r2);
                if v1 != v2 {
                    self.advance_pc();
                }
            },
            Chip8Opcode::LeftShiftRegisterByRegister(r1, r2) => {
                let v2 = self.get_register(r2);
                let msb = (v2 & 0x80) >> 7;
                let value = v2 << 1;

                self.set_register(r1, value);
                self.set_register(r2, value);

                // Set VF to the most significant bit of v2 before the shift
                self.set_register(0xf, msb);
            },
            // TODO: Call Sub... lots more
            Chip8Opcode::Random(target_register, value) => {
                self.set_register(target_register, rand::random::<u8>() & value);
            },
            // TODO: Draw... lots more
            Chip8Opcode::JumpFromV0(offset) => {
                let base = self.get_register(0) as u16; // v0
                self.program_counter = offset + base; // TODO: make sure that we don't skip the first instruction here
            },
            Chip8Opcode::SetDelayTimer(target_register) => {
                let value = self.get_register(target_register);
                self.delay_timer = value;
            },
            Chip8Opcode::ReadDelayTimer(destination_register) => {
                let timer = self.delay_timer;
                self.set_register(destination_register, timer);
            },
            Chip8Opcode::SetSoundTimer(target_register) => {
                let value = self.get_register(target_register);
                self.sound_timer = value;
            },
            Chip8Opcode::AddToIndexRegister(target_register) => {
                let value = self.get_register(target_register);
                self.index += value as u16; // any special overflow conditions?
            },
            // TODO: use sprite, etc.
            _ => panic!(
                "pc={} Not implemented yet: '{:?}'",
                self.program_counter, op
            ),
        }
    }

    pub fn step(&mut self) {
        // fetch
        let pc: usize = self.program_counter as usize;
        let instruction = (self.memory[pc] as u16) << 8 | (self.memory[pc + 1] as u16);

        // advance pointer to next instruction (execute may change address)
        self.program_counter += 2;

        // decode
        let decoded = self.decode(instruction);

        // execute
        self.execute(decoded);

        // update timers (60Hz - need timing)
    }

    fn get_register(&self, register_index: Chip8Register) -> Chip8Value {
        self.registers[register_index as usize]
    }

    fn set_register(&mut self, register_index: Chip8Register, register_value: Chip8Value) {
        self.registers[register_index as usize] = register_value;
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;

    let mut chip8 = ComputerState::new();

    chip8.load_program("roms/c8games/PONG");

    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();

        chip8.step();
        // TODO: Draw contents of memory
        // TODO: Set keymap state
        // TODO: run this inner loop only 60 hz

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000u32 / 60));
    }
}

#[cfg(test)]
mod computer_tests {
    use ComputerState;
    use opcodes::Chip8Opcode;

    fn top_nibble(instruction: u16) -> u8 {
        ((instruction & 0xf000) >> 8) as u8
    }

    fn new_test_emulator() -> ComputerState {
        ComputerState::new()
    }

    fn test_decode(instruction: u16) -> Chip8Opcode {
        let computer = new_test_emulator();
        computer.decode(instruction)
    }

    #[test]
    fn top_nibble_works() {
        // sanity test
        assert_eq!(top_nibble(0x0fff), 0x00);
        assert_eq!(top_nibble(0x1fff), 0x10);
        assert_eq!(top_nibble(0x2fff), 0x20);
    }

    #[test]
    fn basic_decodes_work() {
        assert_eq!(test_decode(0x0abc), Chip8Opcode::Call(0xabc));
        assert_eq!(test_decode(0x00e0), Chip8Opcode::DisplayClear);
        assert_eq!(test_decode(0x00ee), Chip8Opcode::ReturnFromSubroutine);
        assert_eq!(test_decode(0x1abc), Chip8Opcode::Goto(0xabc));
        assert_eq!(test_decode(0x2abc), Chip8Opcode::CallSub(0xabc));
        assert_eq!(test_decode(0x3abc), Chip8Opcode::SkipNextIfEqual(0xa, 0xbc));
        assert_eq!(test_decode(0x4abc), Chip8Opcode::SkipNextIfNotEqual(0xa, 0xbc));
        assert_eq!(test_decode(0x5ab0), Chip8Opcode::SkipNextIfRegistersEqual(0xa, 0xb));
        assert_eq!(test_decode(0x6a14), Chip8Opcode::SetRegister(0xa, 0x14));

        assert_eq!(test_decode(0x8ab0), Chip8Opcode::SetRegisterToRegister(0xa, 0xb));
        assert_eq!(test_decode(0x8ab1), Chip8Opcode::RegisterRegisterOr(0xa, 0xb));
        assert_eq!(test_decode(0x8ab2), Chip8Opcode::RegisterRegisterAnd(0xa, 0xb));
        assert_eq!(test_decode(0x8ab3), Chip8Opcode::RegisterRegisterXor(0xa, 0xb));
        assert_eq!(test_decode(0x8ab4), Chip8Opcode::IncrementRegisterWithRegister(0xa, 0xb));
        assert_eq!(test_decode(0x8ab5), Chip8Opcode::DecrementRegisterWithRegister(0xa, 0xb));
        assert_eq!(test_decode(0x8ab6), Chip8Opcode::ShiftRegisterByRegister(0xa, 0xb));
        assert_eq!(test_decode(0x8ab7), Chip8Opcode::YRegisterMinusXRegister(0xa, 0xb));
        assert_eq!(test_decode(0x8abe), Chip8Opcode::LeftShiftRegisterByRegister(0xa, 0xb));

        assert_eq!(test_decode(0x9ab0), Chip8Opcode::SkipNextIfRegistersNotEqual(0xa, 0xb));

        assert_eq!(test_decode(0xabcd), Chip8Opcode::SetIndexRegister(0xbcd));
        assert_eq!(test_decode(0xbabc), Chip8Opcode::JumpFromV0(0xabc));
        assert_eq!(test_decode(0xcabc), Chip8Opcode::Random(0xa, 0xbc));
        assert_eq!(test_decode(0xdabc), Chip8Opcode::Draw(0xa, 0xb, 0xc));
        assert_eq!(test_decode(0xe19e), Chip8Opcode::SkipNextIfKeyDown(1));
        assert_eq!(test_decode(0xe1a1), Chip8Opcode::SkipNextIfKeyUp(1));

        // Extended opcodes
        assert_eq!(test_decode(0xfa07), Chip8Opcode::ReadDelayTimer(0xa));
        assert_eq!(test_decode(0xfa0a), Chip8Opcode::BlockOnKeyPress(0xa));
        assert_eq!(test_decode(0xfa15), Chip8Opcode::SetDelayTimer(0xa));
        assert_eq!(test_decode(0xfa18), Chip8Opcode::SetSoundTimer(0xa));
        assert_eq!(test_decode(0xfa1e), Chip8Opcode::AddToIndexRegister(0xa));
        assert_eq!(test_decode(0xfa29), Chip8Opcode::UseSprite(0xa));
        assert_eq!(test_decode(0xfa33), Chip8Opcode::ReadRegisterAsBCD(0xa));
        assert_eq!(test_decode(0xfa55), Chip8Opcode::DumpRegisters(0xa));
        assert_eq!(test_decode(0xfa65), Chip8Opcode::FillRegisters(0xa));
    }

    #[test]
    #[should_panic]
    fn mangled_keydown_decode_panics() {
        test_decode(0xe3ff); // 0xff is not a valid discriminating byte, so it should bail
    }

    #[test]
    #[should_panic]
    fn mangled_skip_next_if_registers_equal_panics() {
        test_decode(0x5ab1); // must end in 0
    }

    #[test]
    #[should_panic]
    fn mangled_alu_panics() {
        test_decode(0x8abf); // must end in 0..7, or E
    }

    #[test]
    #[should_panic]
    fn mangled_skip_next_if_registers_not_equal_panics() {
        test_decode(0x9ab1); // must end in 0
    }

    #[test]
    #[should_panic]
    fn mangled_extended_op_panics() {
        test_decode(0xfabf); // must end in 07, 09, etc. not BF
    }

    // Execute tests -------

    #[test]
    fn skip_next_if_equal_works() {
        let mut computer = new_test_emulator();
        let original_pc = computer.program_counter;

        computer.set_register(0, 66);
        computer.execute(Chip8Opcode::SkipNextIfEqual(0, 67));

        // pc should not change if values not equal
        assert_eq!(computer.program_counter, original_pc);

        computer.execute(Chip8Opcode::SkipNextIfEqual(0, 66));

        // pc should advance past the next instruction if equal
        assert_eq!(computer.program_counter, original_pc + 2);
    }

    #[test]
    fn skip_next_if_not_equal_works() {
        let mut computer = new_test_emulator();
        let original_pc = computer.program_counter;

        computer.set_register(0, 66);
        computer.execute(Chip8Opcode::SkipNextIfNotEqual(0, 66));

        // pc should not change if values are equal
        assert_eq!(computer.program_counter, original_pc);

        computer.execute(Chip8Opcode::SkipNextIfNotEqual(0, 67));

        // skip over next instruction if values not equal
        assert_eq!(computer.program_counter, original_pc + 2);
    }

    #[test]
    fn skip_next_if_registers_equal_works() {
        let mut computer = new_test_emulator();
        let original_pc = computer.program_counter;

        computer.set_register(0, 66);
        computer.set_register(1, 99);
        computer.set_register(2, 66);

        // 66 != 99
        computer.execute(Chip8Opcode::SkipNextIfRegistersEqual(0, 1));
        assert_eq!(computer.program_counter, original_pc);

        // 66 == 66
        computer.execute(Chip8Opcode::SkipNextIfRegistersEqual(0, 2));
        assert_eq!(computer.program_counter, original_pc + 2);
    }

    #[test]
    fn skip_next_if_registers_not_equal_works() {
        let mut computer = new_test_emulator();
        let original_pc = computer.program_counter;

        computer.set_register(0, 66);
        computer.set_register(1, 99);
        computer.set_register(2, 66);

        computer.execute(Chip8Opcode::SkipNextIfRegistersNotEqual(0, 2));

        // 66 == 66
        assert_eq!(computer.program_counter, original_pc);

        computer.execute(Chip8Opcode::SkipNextIfRegistersNotEqual(0, 1));

        // 66 != 99
        assert_eq!(computer.program_counter, original_pc + 2);
    }

    #[test]
    fn regular_increment_works() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 150);
        computer.execute(Chip8Opcode::IncrementRegister(0, 10));

        assert_eq!(computer.get_register(0), 160);
    }

    #[test]
    fn regular_increment_wraps_overflow() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 250);
        computer.execute(Chip8Opcode::IncrementRegister(0, 10));

        assert_eq!(computer.get_register(0), 4);
    }

    #[test]
    fn reg_reg_addition_works() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 25);
        computer.set_register(1, 10);
        computer.execute(Chip8Opcode::IncrementRegisterWithRegister(0, 1));

        assert_eq!(computer.get_register(0), 35);
        assert_eq!(computer.get_register(1), 10);
    }

    #[test]
    fn reg_reg_addition_wraps_overflow() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 255);
        computer.set_register(1, 10);

        computer.execute(Chip8Opcode::IncrementRegisterWithRegister(0, 1));

        // overflow should wrap, not crash
        assert_eq!(computer.get_register(0), 9);
        assert_eq!(computer.get_register(1), 10); // make sure reg y is not touched
    }

    #[test]
    fn reg_reg_addition_sets_carry_flag() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 200);
        computer.set_register(1, 10);

        computer.execute(Chip8Opcode::IncrementRegisterWithRegister(0, 1));
        assert_eq!(computer.get_register(0xf), 0); // carry flag must not be set for non-overflow

        computer.set_register(2, 255);
        computer.execute(Chip8Opcode::IncrementRegisterWithRegister(2, 1));
        assert_eq!(computer.get_register(0xf), 1); // did overflow, so carry flag must be set
    }

    #[test]
    fn reg_reg_decrement_works() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 17);
        computer.set_register(1, 2);
        computer.execute(Chip8Opcode::DecrementRegisterWithRegister(0, 1));

        assert_eq!(computer.get_register(0), 15);
        assert_eq!(computer.get_register(1), 2);
    }

    #[test]
    fn reg_reg_decrement_wraps_underflow() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 0);
        computer.set_register(1, 2);
        computer.execute(Chip8Opcode::DecrementRegisterWithRegister(0, 1));

        assert_eq!(computer.get_register(0), 254);
        assert_eq!(computer.get_register(1), 2);
    }

    #[test]
    fn reg_reg_decrement_sets_borrow_register() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 100);
        computer.set_register(1, 1);
        computer.execute(Chip8Opcode::DecrementRegisterWithRegister(0, 1));
        assert_eq!(computer.get_register(0xf), 1); // NOT borrowed

        computer.set_register(2, 150);
        computer.execute(Chip8Opcode::DecrementRegisterWithRegister(0, 2));
        assert_eq!(computer.get_register(0xf), 0); // did borrow
    }

    #[test]
    fn reg_reg_shift_works() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 8);
        computer.set_register(1, 32);

        computer.execute(Chip8Opcode::ShiftRegisterByRegister(0, 1));

        assert_eq!(computer.get_register(0), 32 >> 1);
        assert_eq!(computer.get_register(1), 32); // remains unchanged
    }

    #[test]
    fn reg_reg_shift_sets_f_register() {
        let mut computer = new_test_emulator();
        computer.set_register(1, 0xff);
        computer.execute(Chip8Opcode::ShiftRegisterByRegister(0, 1));
        assert_eq!(computer.get_register(0xf), 1); // least significant bit of 0xff is 1

        computer.set_register(1, 0x01);
        computer.execute(Chip8Opcode::ShiftRegisterByRegister(0, 1));
        assert_eq!(computer.get_register(0xf), 1); // least significant bit of 0x01 is also 1

        computer.set_register(1, 0x00);
        computer.execute(Chip8Opcode::ShiftRegisterByRegister(0, 1));
        assert_eq!(computer.get_register(0xf), 0); // least significant bit of 0x00 is 0

        computer.set_register(1, 0xf0);
        computer.execute(Chip8Opcode::ShiftRegisterByRegister(0, 1));
        assert_eq!(computer.get_register(0xf), 0); // least significant bit of 0xf0 is also 1
    }

    #[test]
    fn reg_reg_left_shift_works() {
        let mut computer = new_test_emulator();
        computer.set_register(1, 60);

        computer.execute(Chip8Opcode::LeftShiftRegisterByRegister(0, 1));
        assert_eq!(computer.get_register(0), 60 << 1);
        assert_eq!(computer.get_register(1), 60 << 1); // should be changed too
    }

    #[test]
    fn reg_reg_left_shift_sets_f_register() {
        let mut computer = new_test_emulator();
        computer.set_register(1, 0xff);
        computer.execute(Chip8Opcode::LeftShiftRegisterByRegister(0, 1));
        assert_eq!(computer.get_register(0xf), 1); // most significant bit was non-zero

        computer.set_register(1, 0x0f);
        computer.execute(Chip8Opcode::LeftShiftRegisterByRegister(0, 1));
        assert_eq!(computer.get_register(0xf), 0); // most significant bit of 0x0f is zero
    }

    #[test]
    fn y_minus_x_works() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 35);
        computer.set_register(1, 100);

        computer.execute(Chip8Opcode::YRegisterMinusXRegister(0, 1));

        assert_eq!(computer.get_register(0), 65); // 100 - 35
    }

    #[test]
    fn y_minus_x_sets_f_register() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 150);
        computer.set_register(1, 200);
        computer.execute(Chip8Opcode::YRegisterMinusXRegister(0, 1));
        assert_eq!(computer.get_register(0), 50);
        assert_eq!(computer.get_register(0xf), 1); // NOT borrowed

        // now underflow
        computer.set_register(0, 25);
        computer.set_register(1, 15);
        computer.execute(Chip8Opcode::YRegisterMinusXRegister(0, 1));
        assert_eq!(computer.get_register(0xf), 0); // borrowed
    }

    #[test]
    fn read_delay_timer_works() {
        let mut computer = new_test_emulator();
        computer.delay_timer = 100;

        computer.execute(Chip8Opcode::ReadDelayTimer(0));
        assert_eq!(computer.get_register(0), 100);
        assert_eq!(computer.delay_timer, 100); // make sure the value is preserved
    }

    #[test]
    fn jump_from_v0_works() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 10);
        computer.execute(Chip8Opcode::JumpFromV0(150));
        assert_eq!(computer.program_counter, 160); // should this be checked for opcode alignment?
    }

    #[test]
    fn set_delay_timer_works() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 123);
        computer.execute(Chip8Opcode::SetDelayTimer(0));
        assert_eq!(computer.delay_timer, 123);
        assert_eq!(computer.get_register(0), 123);
    }

    #[test]
    fn set_sound_timer_works() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 124);
        computer.execute(Chip8Opcode::SetSoundTimer(0));
        assert_eq!(computer.sound_timer, 124);
        assert_eq!(computer.get_register(0), 124)
    }

    #[test]
    fn register_register_or_works() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 0xa0);
        computer.set_register(1, 0x0f);
        computer.execute(Chip8Opcode::RegisterRegisterOr(0, 1));
        assert_eq!(computer.get_register(0), 0xaf);
        assert_eq!(computer.get_register(1), 0x0f); // make sure the y-register is not touched
    }

    #[test]
    fn register_register_and_works() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 0xaf);
        computer.set_register(1, 0x0f);
        computer.execute(Chip8Opcode::RegisterRegisterAnd(0, 1));
        assert_eq!(computer.get_register(0), 0x0f);
        assert_eq!(computer.get_register(1), 0x0f); // make sure the y-register is not touched
    }

    #[test]
    fn register_register_xor_works() {
        let mut computer = new_test_emulator();
        computer.set_register(0, 0xff);
        computer.set_register(1, 0x0f);
        computer.execute(Chip8Opcode::RegisterRegisterXor(0, 1));
        assert_eq!(computer.get_register(0), 0xf0);
        assert_eq!(computer.get_register(1), 0x0f); // make sure the y-register is not touched
    }
}
