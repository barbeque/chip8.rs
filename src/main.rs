extern crate sdl2;
extern crate rand;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::path::PathBuf;
use std::time::Duration;
use rand::Rng;

mod opcodes;
use opcodes::*;

struct ComputerState {
    // The current opcode being decoded
    opcode: u16,
    // 4K main memory
    memory: [u8; 4096],
    // General purpose registers V0..VE
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
            opcode: 0,
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

        // TODO: load ROM contents (font set)
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
        println!("Loading CHIP-8 program '{}'", target_path)
    }

    pub fn decode(&self, instruction: u16) -> Chip8Opcode {
        // Instructions are stored big-endian so we're good
        let top_nibble = (instruction & 0xf000) >> 12;
        if(top_nibble == 0x0) {
            // CALL
        }
        else if(top_nibble == 0x1) {
            // 1NNN - jump
            return Chip8Opcode::Goto(instruction & 0xfff);
        }
        else if(top_nibble == 0x2) {
            // 2NNN - call sub at NNN
            return Chip8Opcode::CallSub(instruction & 0xfff);
        }
        else if(top_nibble == 0x3) {
        }
        else if(top_nibble == 0x4) {

        }
        else if(top_nibble == 0x5) {

        }
        else if(top_nibble == 0x6) {
            // assign
        }
        else if(top_nibble == 0x7) {
            // increment w/o carry
        }
        else if(top_nibble == 0x8) {
            let bottom_nibble = (instruction & 0x000f);
        }
        else if(top_nibble == 0x9) {
            // skip next if Vx != Vy
        }
        else if(top_nibble == 0xa) {
            // set index
        }
        else if(top_nibble == 0xb) {
            // far jump
        }
        else if(top_nibble == 0xc) {
            // random
        }
        else if(top_nibble == 0xd) {
            // draw sprite
        }
        else if(top_nibble == 0xe) {
            // key operations depending on bottom byte
            let bottom_byte = (instruction & 0xff);
            if(bottom_byte == 0x9e) {
                // skip if key stored in Vx is pressed
            }
            else if(bottom_byte == 0xa1) {
                // skip if key stored in Vx is not pressed
            }
            else {
                panic!("Malformed key press instruction {:x}", instruction);
            }
        }
        else if(top_nibble == 0xf) {
            // timer ops...
            let bottom_byte = (instruction & 0xff);
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
            // TODO: Call Sub... lots more
            Chip8Opcode::Random(target_register, value) => {
                self.set_register(target_register, rand::random::<u8>() & value);
            },
            // TODO: Draw... lots more
            Chip8Opcode::SetDelayTimer(target_register) => {
                let value = self.get_register(target_register);
                self.delay_timer = value;
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
    fn top_nibble(instruction: u16) -> u8 {
        ((instruction & 0xf000) >> 8) as u8
    }

    #[test]
    fn top_nibble_works() {
        // sanity test
        assert_eq!(top_nibble(0x0fff), 0x00);
        assert_eq!(top_nibble(0x1fff), 0x10);
        assert_eq!(top_nibble(0x2fff), 0x20);
    }
}
