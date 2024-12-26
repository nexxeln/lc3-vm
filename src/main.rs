use lc3_vm::{
    terminal::Terminal,
    vm::machine::{VM, VMError},
};
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::process;

fn run() -> Result<(), VMError> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} [image-file1] ...", args[0]);
        process::exit(2);
    }

    // initialize vm with terminal
    let terminal = Terminal::new()?;
    let mut vm = VM::new(terminal);

    // load all program images
    for image_path in &args[1..] {
        vm.load_program(image_path)?;
    }

    // set up signal handling
    let mut signals = Signals::new(&[SIGINT])?;
    std::thread::spawn(move || {
        for _ in signals.forever() {
            eprintln!("\nReceived SIGINT");
            process::exit(-2);
        }
    });

    // run the vm
    vm.run()
}

fn main() {
    match run() {
        Ok(_) => process::exit(0),
        Err(VMError::IO(err)) => {
            eprintln!("IO Error: {}", err);
            process::exit(1);
        }
        Err(VMError::InvalidOpCode(op)) => {
            eprintln!("Invalid opcode: {:#x}", op);
            process::exit(1);
        }
        Err(VMError::InvalidProgram) => {
            eprintln!("Invalid program format");
            process::exit(1);
        }
    }
}
