# chip8.rs
![build status](https://travis-ci.org/barbeque/chip8.rs.svg?branch=master)

chip8.rs is an emulator for the [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) interpreted game language, written in Rust.

## Building
### macOS
 1. Install the SDL2 framework [from the libsdl website](https://www.libsdl.org/download-2.0.php). Future versions may require SDL2_ttf.
 2. Make sure that your Rust build environment is up to date.
 3. `cargo run` should build and launch the emulator with a default game.

### Linux
 1. Install SDL2 development packages through apt, or whatever your package manager is.
 2. `cargo run` should build and launch the emulator with a default game.

## Usage
Pass the path to a CHIP-8 ROM to load that ROM instead of the default game. Some public-domain example games are included in the `roms/c8games` directory.

## Screenshots
![HIDDEN game](/screenshots/chip8-hidden.png)
![BRIX game](/screenshots/chip8-brix.png)
![TETRIS game](/screenshots/chip8-tetris.png)
