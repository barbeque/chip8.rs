extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

struct ComputerState {
    // The current opcode being decoded
    opcode : u16,
    // 4K main memory
    memory : [u8; 4096],
    // General purpose registers V0..VE
    registers : [u8; 16],
    // Index register
    index : u16,
    // Program counter
    program_counter : u16,
    // 2K Video memory
    gfx : [u8; 64 * 32],
    // Delay timer
    delay_timer : u8,
    // Sound timer
    sound_timer : u8,
    // Stack
    stack : [u16; 16],
    // Stack pointer
    sp : u16,
    // Keyboard state
    keys : [u8; 16]
}

impl ComputerState {
    pub fn new() -> ComputerState {
        ComputerState{
            opcode: 0,
            memory: [0u8; 4096],
            registers: [0u8; 16],
            index: 0,
            program_counter: 0,
            gfx: [0u8; (64 * 32)],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0u16; 16],
            sp: 0,
            keys: [0u8; 16]
        }
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
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

    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000u32 / 60));
    }
}
