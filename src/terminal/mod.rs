mod unix;
pub use unix::Terminal;

pub trait TerminalInterface {
    fn disable_input_buffering(&mut self) -> std::io::Result<()>;
    fn restore_input_buffering(&self) -> std::io::Result<()>;
}
