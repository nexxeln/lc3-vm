use super::{memory::Memory, ops::OpCode, register::Register};
use crate::terminal::TerminalInterface;
use std::fs::File;
use std::io::{self, Read, Write};

pub struct VM<T: TerminalInterface> {
    memory: Memory,
    registers: [u16; Register::COUNT as usize],
    terminal: T,
}

#[derive(Debug)]
pub enum VMError {
    IO(io::Error),
    InvalidOpCode(u16),
    InvalidProgram,
}

impl From<io::Error> for VMError {
    fn from(error: io::Error) -> Self {
        VMError::IO(error)
    }
}

impl<T: TerminalInterface> VM<T> {
    pub fn new(terminal: T) -> Self {
        Self {
            memory: Memory::new(),
            registers: [0; Register::COUNT as usize],
            terminal,
        }
    }

    pub fn load_program(&mut self, path: &str) -> Result<(), VMError> {
        let mut file = File::open(path)?;

        // read origin
        let mut origin_bytes = [0u8; 2];
        file.read_exact(&mut origin_bytes)?;
        let origin = u16::from_be_bytes(origin_bytes);

        // read program
        let mut program = Vec::new();
        file.read_to_end(&mut program)?;

        let words = program
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();

        self.memory.load_image(origin, &words);
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        self.terminal.disable_input_buffering()?;
        let result = self.execute();
        self.terminal.restore_input_buffering()?;
        result
    }

    fn execute(&mut self) -> Result<(), VMError> {
        // set pc to starting position (0x3000 is the default)
        const PC_START: u16 = 0x3000;
        self.registers[Register::PC as usize] = PC_START;

        // set condition flag to z
        self.registers[Register::COND as usize] = super::register::FL_ZRO;

        let mut running = true;
        while running {
            // fetch
            let pc = self.registers[Register::PC as usize];
            self.registers[Register::PC as usize] += 1;
            let instr = self.memory.read(pc);

            // decode
            let op_code = (instr >> 12) as u16;

            // execute
            match OpCode::from_u16(op_code) {
                Some(op) => {
                    if let OpCode::TRAP = op {
                        if (instr & 0xFF) == super::ops::TRAP_HALT {
                            running = false;
                        }
                    }
                    self.execute_instruction(op, instr)?
                }
                None => return Err(VMError::InvalidOpCode(op_code)),
            }
        }
        Ok(())
    }

    fn execute_instruction(&mut self, op: OpCode, instr: u16) -> Result<(), VMError> {
        match op {
            OpCode::ADD => self.add_op(instr),
            OpCode::AND => self.and_op(instr),
            OpCode::NOT => self.not_op(instr),
            OpCode::BR => self.branch_op(instr),
            OpCode::JMP => self.jump_op(instr),
            OpCode::JSR => self.jsr_op(instr),
            OpCode::LD => self.load_op(instr),
            OpCode::LDI => self.load_indirect_op(instr),
            OpCode::LDR => self.load_register_op(instr),
            OpCode::LEA => self.load_effective_address_op(instr),
            OpCode::ST => self.store_op(instr),
            OpCode::STI => self.store_indirect_op(instr),
            OpCode::STR => self.store_register_op(instr),
            OpCode::TRAP => self.trap_op(instr),
            OpCode::RES | OpCode::RTI => Err(VMError::InvalidOpCode(op as u16)),
        }
    }

    fn add_op(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0x7;
        let r1 = (instr >> 6) & 0x7;
        let imm_flag = (instr >> 5) & 0x1;

        self.registers[r0 as usize] = if imm_flag != 0 {
            let imm5 = sign_extend(instr & 0x1F, 5);
            self.registers[r1 as usize].wrapping_add(imm5)
        } else {
            let r2 = instr & 0x7;
            self.registers[r1 as usize].wrapping_add(self.registers[r2 as usize])
        };

        self.update_flags(r0);
        Ok(())
    }

    fn and_op(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0x7;
        let r1 = (instr >> 6) & 0x7;
        let imm_flag = (instr >> 5) & 0x1;

        self.registers[r0 as usize] = if imm_flag != 0 {
            let imm5 = sign_extend(instr & 0x1F, 5);
            self.registers[r1 as usize] & imm5
        } else {
            let r2 = instr & 0x7;
            self.registers[r1 as usize] & self.registers[r2 as usize]
        };

        self.update_flags(r0);
        Ok(())
    }

    fn not_op(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0x7;
        let r1 = (instr >> 6) & 0x7;

        self.registers[r0 as usize] = !self.registers[r1 as usize];
        self.update_flags(r0);
        Ok(())
    }

    fn branch_op(&mut self, instr: u16) -> Result<(), VMError> {
        let pc_offset = sign_extend(instr & 0x1FF, 9);
        let cond_flag = (instr >> 9) & 0x7;

        if (cond_flag & self.registers[Register::COND as usize]) != 0 {
            self.registers[Register::PC as usize] =
                self.registers[Register::PC as usize].wrapping_add(pc_offset);
        }
        Ok(())
    }

    fn jump_op(&mut self, instr: u16) -> Result<(), VMError> {
        let r1 = (instr >> 6) & 0x7;
        self.registers[Register::PC as usize] = self.registers[r1 as usize];
        Ok(())
    }

    fn jsr_op(&mut self, instr: u16) -> Result<(), VMError> {
        let long_flag = (instr >> 11) & 1;
        self.registers[Register::R7 as usize] = self.registers[Register::PC as usize];

        if long_flag != 0 {
            let long_pc_offset = sign_extend(instr & 0x7FF, 11);
            self.registers[Register::PC as usize] =
                self.registers[Register::PC as usize].wrapping_add(long_pc_offset);
        } else {
            let r1 = (instr >> 6) & 0x7;
            self.registers[Register::PC as usize] = self.registers[r1 as usize];
        }
        Ok(())
    }

    fn load_op(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0x7;
        let pc_offset = sign_extend(instr & 0x1FF, 9);
        let addr = self.registers[Register::PC as usize].wrapping_add(pc_offset);

        self.registers[r0 as usize] = self.memory.read(addr);
        self.update_flags(r0);
        Ok(())
    }

    fn load_indirect_op(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0x7;
        let pc_offset = sign_extend(instr & 0x1FF, 9);
        let indirect_addr = self
            .memory
            .read(self.registers[Register::PC as usize].wrapping_add(pc_offset));

        self.registers[r0 as usize] = self.memory.read(indirect_addr);
        self.update_flags(r0);
        Ok(())
    }

    fn load_register_op(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0x7;
        let r1 = (instr >> 6) & 0x7;
        let offset = sign_extend(instr & 0x3F, 6);

        let addr = self.registers[r1 as usize].wrapping_add(offset);
        self.registers[r0 as usize] = self.memory.read(addr);
        self.update_flags(r0);
        Ok(())
    }

    fn load_effective_address_op(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0x7;
        let pc_offset = sign_extend(instr & 0x1FF, 9);

        self.registers[r0 as usize] = self.registers[Register::PC as usize].wrapping_add(pc_offset);
        self.update_flags(r0);
        Ok(())
    }

    fn store_op(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0x7;
        let pc_offset = sign_extend(instr & 0x1FF, 9);
        let addr = self.registers[Register::PC as usize].wrapping_add(pc_offset);

        self.memory.write(addr, self.registers[r0 as usize]);
        Ok(())
    }

    fn store_indirect_op(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0x7;
        let pc_offset = sign_extend(instr & 0x1FF, 9);
        let indirect_addr = self
            .memory
            .read(self.registers[Register::PC as usize].wrapping_add(pc_offset));

        self.memory
            .write(indirect_addr, self.registers[r0 as usize]);
        Ok(())
    }

    fn store_register_op(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0x7;
        let r1 = (instr >> 6) & 0x7;
        let offset = sign_extend(instr & 0x3F, 6);

        let addr = self.registers[r1 as usize].wrapping_add(offset);
        self.memory.write(addr, self.registers[r0 as usize]);
        Ok(())
    }

    fn trap_op(&mut self, instr: u16) -> Result<(), VMError> {
        use super::ops::{TRAP_GETC, TRAP_HALT, TRAP_IN, TRAP_OUT, TRAP_PUTS, TRAP_PUTSP};

        self.registers[Register::R7 as usize] = self.registers[Register::PC as usize];

        match instr & 0xFF {
            TRAP_GETC => {
                let mut buffer = [0u8; 1];
                io::stdin().read_exact(&mut buffer)?;
                self.registers[Register::R0 as usize] = buffer[0] as u16;
                self.update_flags(Register::R0 as u16);
            }
            TRAP_OUT => {
                let char = (self.registers[Register::R0 as usize] & 0xFF) as u8 as char;
                print!("{}", char);
                io::stdout().flush()?;
            }
            TRAP_PUTS => {
                let mut addr = self.registers[Register::R0 as usize];
                while self.memory.read(addr) != 0 {
                    let char = (self.memory.read(addr) & 0xFF) as u8 as char;
                    print!("{}", char);
                    addr += 1;
                }
                io::stdout().flush()?;
            }
            TRAP_IN => {
                print!("Enter a character: ");
                io::stdout().flush()?;
                let mut buffer = [0u8; 1];
                io::stdin().read_exact(&mut buffer)?;
                let char = buffer[0] as char;
                print!("{}", char);
                io::stdout().flush()?;
                self.registers[Register::R0 as usize] = char as u16;
                self.update_flags(Register::R0 as u16);
            }
            TRAP_PUTSP => {
                let mut addr = self.registers[Register::R0 as usize];
                while self.memory.read(addr) != 0 {
                    let value = self.memory.read(addr);
                    let char1 = (value & 0xFF) as u8 as char;
                    print!("{}", char1);

                    let char2 = (value >> 8) as u8;
                    if char2 != 0 {
                        print!("{}", char2 as char);
                    }
                    addr += 1;
                }
                io::stdout().flush()?;
            }
            TRAP_HALT => {
                println!("HALT");
                io::stdout().flush()?;
                return Ok(());
            }
            unknown => return Err(VMError::InvalidOpCode(unknown)),
        }
        Ok(())
    }

    fn update_flags(&mut self, r: u16) {
        let val = self.registers[r as usize];
        let flag = if val == 0 {
            super::register::FL_ZRO
        } else if (val >> 15) != 0 {
            super::register::FL_NEG
        } else {
            super::register::FL_POS
        };
        self.registers[Register::COND as usize] = flag;
    }
}

fn sign_extend(x: u16, bit_count: u16) -> u16 {
    if (x >> (bit_count - 1)) & 1 != 0 {
        x | (0xFFFF << bit_count)
    } else {
        x
    }
}
