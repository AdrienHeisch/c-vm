use clap::Parser;
use std::{fs, path::PathBuf};

mod interpreter;

#[derive(Parser)]
struct Args {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    file: PathBuf,
}

fn main() {
    let args = Args::parse();
    let bytes = fs::read(args.file).unwrap();
    interpreter::interpret(&bytes);
}
