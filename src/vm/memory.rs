use std::io::{self, Read};

pub const MEMORY_MAX: usize = 1 << 16; // 65536 locations

// memory-mapped registers
pub const MR_KBSR: u16 = 0xFE00; // keyboard status
pub const MR_KBDR: u16 = 0xFE02; // keyboard data

pub struct Memory {
    cells: [u16; MEMORY_MAX],
}

impl Memory {
    pub fn new() -> Self {
        Self {
            cells: [0; MEMORY_MAX],
        }
    }

    pub fn write(&mut self, address: u16, value: u16) {
        self.cells[address as usize] = value;
    }

    pub fn read(&mut self, address: u16) -> u16 {
        if address == MR_KBSR {
            if Self::check_key() {
                self.cells[MR_KBSR as usize] = 1 << 15;
                self.cells[MR_KBDR as usize] =
                    io::stdin().bytes().next().and_then(|b| b.ok()).unwrap_or(0) as u16;
            } else {
                self.cells[MR_KBSR as usize] = 0;
            }
        }
        self.cells[address as usize]
    }

    fn check_key() -> bool {
        use nix::sys::select::{FdSet, select};
        use nix::sys::time::TimeVal;

        let mut readfds = FdSet::new();
        readfds.insert(0); // stdin

        let mut timeout = TimeVal::new(0, 0);
        match select(1, Some(&mut readfds), None, None, Some(&mut timeout)) {
            Ok(n) => n > 0,
            Err(_) => false,
        }
    }

    pub fn load_image(&mut self, origin: u16, program: &[u16]) {
        let start = origin as usize;
        let end = start + program.len();
        self.cells[start..end].copy_from_slice(program);
    }
}
