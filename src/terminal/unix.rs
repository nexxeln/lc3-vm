use super::TerminalInterface;
use std::io;
use termios::{ECHO, ICANON, TCSANOW, Termios, tcsetattr};

pub struct Terminal {
    original_tio: Termios,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        let original_tio = Termios::from_fd(0)?;
        Ok(Self { original_tio })
    }
}

impl TerminalInterface for Terminal {
    fn disable_input_buffering(&mut self) -> io::Result<()> {
        let mut new_tio = self.original_tio.clone();
        new_tio.c_lflag &= !(ICANON | ECHO);
        tcsetattr(0, TCSANOW, &new_tio)?;
        Ok(())
    }

    fn restore_input_buffering(&self) -> io::Result<()> {
        tcsetattr(0, TCSANOW, &self.original_tio)?;
        Ok(())
    }
}
