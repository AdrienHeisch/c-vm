use clap::Parser;
use loader::load;
use std::{fs, path::PathBuf};

mod instruction;
mod interpreter;
mod loader;
mod registers;
mod tui;

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
    tui: bool,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    let bytes = fs::read(args.file)?;
    let program = load(&bytes);
    if args.tui {
        tui::start()?;
    } else {
        interpreter::interpret(&program);
    }
    Ok(())
}
