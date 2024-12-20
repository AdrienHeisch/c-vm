use clap::Parser;
use loader::load;
use std::{fs, path::PathBuf};

#[cfg(feature = "debugger")]
mod debugger;

mod instruction;
mod loader;
mod macros;
mod registers;
mod vm;

#[allow(non_camel_case_types)]
type uvm = u64;

const REG_LEN: usize = uvm::BITS as usize / 8;

#[derive(Parser)]
struct Args {
    /// Sets a custom target file
    #[arg(short, long, value_name = "FILE")]
    file: PathBuf,

    /// Sets a custom config file
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    let bytes = fs::read(args.file)?;
    let program = load(&bytes);

    if args.debug {
        #[cfg(feature = "debugger")]
        debugger::run(&program)?;
        #[cfg(not(feature = "debugger"))]
        println!("Debugger not included in this build");
    } else {
        vm::run(&program);
    }

    Ok(())
}
