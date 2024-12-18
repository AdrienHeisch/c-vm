use clap::Parser;
use loader::load;
use std::{fs, path::PathBuf};

mod instruction;
mod interpreter;
mod loader;

#[allow(non_camel_case_types)]
type uvm = u64;

const REG_LEN: usize = uvm::BITS as usize / 8;

#[derive(Parser)]
struct Args {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    file: PathBuf,
}

fn main() {
    let args = Args::parse();
    let bytes = fs::read(args.file).unwrap();
    let program = load(&bytes);
    interpreter::interpret(&program);
}
