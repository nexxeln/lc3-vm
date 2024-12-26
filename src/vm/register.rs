#[derive(Clone, Copy, Debug)]
pub enum Register {
    R0 = 0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    PC,   // program counter
    COND, // condition flag
    COUNT,
}

// condition flags
pub const FL_POS: u16 = 1 << 0; // positive
pub const FL_ZRO: u16 = 1 << 1; // zero
pub const FL_NEG: u16 = 1 << 2; // negative

impl Register {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Register::R0),
            1 => Some(Register::R1),
            2 => Some(Register::R2),
            3 => Some(Register::R3),
            4 => Some(Register::R4),
            5 => Some(Register::R5),
            6 => Some(Register::R6),
            7 => Some(Register::R7),
            8 => Some(Register::PC),
            9 => Some(Register::COND),
            _ => None,
        }
    }
}
