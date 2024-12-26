#[derive(Clone, Copy, Debug)]
pub enum OpCode {
    BR = 0, // branch
    ADD,    // add
    LD,     // load
    ST,     // store
    JSR,    // jump register
    AND,    // bitwise and
    LDR,    // load register
    STR,    // store register
    RTI,    // unused
    NOT,    // bitwise not
    LDI,    // load indirect
    STI,    // store indirect
    JMP,    // jump
    RES,    // reserved (unused)
    LEA,    // load effective address
    TRAP,   // execute trap
}

// TRAP codes
pub const TRAP_GETC: u16 = 0x20; // get character from keyboard, not echoed onto the terminal
pub const TRAP_OUT: u16 = 0x21; // output a character
pub const TRAP_PUTS: u16 = 0x22; // output a word string
pub const TRAP_IN: u16 = 0x23; // get character from keyboard, echoed onto the terminal
pub const TRAP_PUTSP: u16 = 0x24; // output a byte string
pub const TRAP_HALT: u16 = 0x25; // halt the program

impl OpCode {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(OpCode::BR),
            1 => Some(OpCode::ADD),
            2 => Some(OpCode::LD),
            3 => Some(OpCode::ST),
            4 => Some(OpCode::JSR),
            5 => Some(OpCode::AND),
            6 => Some(OpCode::LDR),
            7 => Some(OpCode::STR),
            8 => Some(OpCode::RTI),
            9 => Some(OpCode::NOT),
            10 => Some(OpCode::LDI),
            11 => Some(OpCode::STI),
            12 => Some(OpCode::JMP),
            13 => Some(OpCode::RES),
            14 => Some(OpCode::LEA),
            15 => Some(OpCode::TRAP),
            _ => None,
        }
    }
}
