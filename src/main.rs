#![warn(
    clippy::pedantic,
    clippy::missing_panics_doc,
    clippy::indexing_slicing
)]
#![allow(clippy::cast_possible_truncation, clippy::unreadable_literal)]

use clap::Parser;
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
    let program = fs::read(args.file)?;

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
