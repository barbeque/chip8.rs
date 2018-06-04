pub type Chip8Address = u16; // Only the bottom 3 nibbles are used
pub type Chip8Register = u8; // There's only 16 of them
pub type Chip8Value = u8;

#[derive(Debug, PartialEq)]
pub enum Chip8Opcode {
    /* 0NNN */ Call(Chip8Address),
    /* 00E0 */ DisplayClear,
    /* 00EE */ ReturnFromSubroutine,
    /* 1NNN */ Goto(Chip8Address),
    /* 2NNN */ CallSub(Chip8Address),
    /* 3XNN */ SkipNextIfEqual(Chip8Register, Chip8Value),
    /* 4XNN */ SkipNextIfNotEqual(Chip8Register, Chip8Value),
    /* 5XY0 */ SkipNextIfRegistersEqual(Chip8Register, Chip8Register),
    /* 6XNN */ SetRegister(Chip8Register, Chip8Value),
    /* 7XNN */ IncrementRegister(Chip8Register, Chip8Value),
    /* 8XY0 */ SetRegisterToRegister(Chip8Register, Chip8Register),
    /* 8XY1 */ RegisterRegisterOr(Chip8Register, Chip8Register),
    /* 8XY2 */ RegisterRegisterAnd(Chip8Register, Chip8Register),
    /* 8XY3 */ RegisterRegisterXor(Chip8Register, Chip8Register),
    /* 8XY4 */ IncrementRegisterWithRegister(Chip8Register, Chip8Register), // Vx += Vy
    /* 8XY5 */ DecrementRegisterWithRegister(Chip8Register, Chip8Register), // Vx -= Vy
    /* 8XY6 */ ShiftRegisterByRegister(Chip8Register, Chip8Register),
    /* 8XY7 */ YRegisterMinusXRegister(/* X */ Chip8Register, /* Y */ Chip8Register), // Vx = Vy - Vx
    /* 8XYE */ LeftShiftRegisterByRegister(Chip8Register, Chip8Register),
    /* 9XY0 */ SkipNextIfRegistersNotEqual(Chip8Register, Chip8Register),
    /* ANNN */ SetIndexRegister(Chip8Address),
    /* BNNN */ JumpFromV0(Chip8Address), // jump to V0 + NNN
    /* CXNN */ Random(Chip8Register, Chip8Value), // Vx = rand() & NN
    /* DXYN */ Draw(Chip8Register, Chip8Register, u8 /* really 4-bit: FIXME */),
    /* EX9E */ SkipNextIfKeyDown(Chip8Register),
    /* EXA1 */ SkipNextIfKeyUp(Chip8Register),
    /* FX07 */ ReadDelayTimer(Chip8Register), // Store in register
    /* FX0A */ BlockOnKeyPress(Chip8Register), // Store in the register when pressed
    /* FX15 */ SetDelayTimer(Chip8Register),
    /* FX18 */ SetSoundTimer(Chip8Register),
    /* FX1E */ AddToIndexRegister(Chip8Register), // I += Vx
    /* FX29 */ UseSprite(Chip8Register), // I = sprites[Vx]
    /* FX33 */ ReadRegisterAsBCD(Chip8Register), // store the BCD of Vx in I
    /* FX55 */ DumpRegisters(Chip8Register), // store V0...Vx in memory starting at I
    /* FX65 */ FillRegisters(Chip8Register), // read from I to V0...Vx
}
