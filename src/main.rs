extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::path::PathBuf;
use std::time::Duration;

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
    program_counter: u16,
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

        if target.exists() {
            let target_path = target.to_str().unwrap();
            println!("Loading CHIP-8 program '{}'", target_path)
        } else {
            // Not found...
            let target_path = target.to_str().unwrap(); // FIXME: might cause a crash if it's a really mangled string. need a safer unwrap.
            panic!(
                "Could not find the file '{}' to load as a CHIP-8 program.",
                target_path
            );
        }
    }

    pub fn decode(&self, instruction: u16) -> Chip8Opcode {
        Chip8Opcode::DumpRegisters(0) // TODO
    }

    pub fn execute(&mut self, op: Chip8Opcode) {
        match op {
            Chip8Opcode::DisplayClear => {
                for i in 0..self.gfx.len() {
                    // FIXME: is a more succinct way to do this?
                    self.gfx[i] = 0;
                }
            },
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
