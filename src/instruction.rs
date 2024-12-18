use std::fmt::Debug;

use crate::{registers::Registers, uvm};

#[derive(Clone, Copy)]
pub struct Instruction {
    pub rfl: bool,
    pub opc: u8,
    pub reg: u8,
    pub val: uvm,
}

const LEN: usize = 4;

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { rfl, opc, reg, val } = self;
        let reg = Registers::register_name(*reg as uvm);
        let val = if *rfl {
            format!("{:<LEN$}", Registers::register_name(*val as uvm))
        } else {
            format!("{val:0>LEN$X}")
        };

        match opc {
            0x00 => write!(f, "NOP           "),
            0x04 => write!(f, "SET    {} {val}", reg),
            0x05 => write!(f, "LOAD   {} {val}", reg),
            0x06 => write!(f, "STORE  {} {val}", reg),
            0x0C => write!(f, "ADD    {} {val}", reg),
            0x0D => write!(f, "SUB    {} {val}", reg),
            0x1C => write!(f, "PUSH      {val}"),
            0x1E => write!(f, "POP    {}     ", reg),
            0x1F => write!(f, "DROP          "),
            0x20 => write!(f, "CALL      {val}"),
            0x21 => write!(f, "RET       {val}"),
            0x22 => write!(f, "JMP       {val}"),
            0x23 => write!(f, "JEQ    {} {val}", reg),
            0x24 => write!(f, "JNE    {} {val}", reg),
            _ => panic!("Unexpected opcode 0x{opc:02X}"),
        }?;
        Ok(())
    }
}
