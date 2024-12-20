use crate::{opc, reg_index, registers::Registers, uvm, REG_LEN};
use std::fmt::Debug;

#[derive(Clone, Copy)]
pub struct Instruction {
    pub rfl: bool,
    pub opc: u8,
    pub reg: u8, // TODO uvm ?
    pub val: uvm,
}

impl Instruction {
    pub fn target_regs(&self) -> (Vec<usize>, Vec<usize>) {
        let Self { rfl, opc, reg, val } = self;
        let (reg, val) = (*reg as usize, *val as usize);
        let (mut dst, mut src) = (Vec::new(), Vec::new());
        if *opc > 0x2C {
            panic!("Unexpected opcode 0x{opc:02X}")
        }
        match opc {
            opc!(CLEAR)
            | opc!(SET)
            | opc!(LOAD)
            | opc!(SWAP)
            | opc!(NEG)
            | opc!(INC)
            | opc!(DEC)
            | opc!(ADD)
            | opc!(SUB)
            | opc!(MUL)
            | opc!(DIV)
            | opc!(MOD)
            | opc!(NOT)
            | opc!(AND)
            | opc!(OR)
            | opc!(XOR)
            | opc!(NAND)
            | opc!(NOR)
            | opc!(NXOR)
            | opc!(SHL)
            | opc!(SHR)
            | opc!(RCL)
            | opc!(RCR)
            | opc!(BSWAP)
            | opc!(POP) => dst.push(reg),
            _ => {}
        }
        match opc {
            opc!(STORE)
            | opc!(CMP)
            | opc!(JEQ)
            | opc!(JNE)
            | opc!(JGT)
            | opc!(JGE)
            | opc!(JLT)
            | opc!(JLE) => src.push(reg),
            _ => {}
        }
        match opc {
            opc!(SWAP) if *rfl => dst.push(val),
            _ => {}
        }
        match opc {
            opc!(HALT)
            | opc!(SYCALL)
            | opc!(SET)
            | opc!(LOAD)
            | opc!(STORE)
            | opc!(CMP)
            | opc!(ADD)
            | opc!(SUB)
            | opc!(MUL)
            | opc!(DIV)
            | opc!(MOD)
            | opc!(AND)
            | opc!(OR)
            | opc!(XOR)
            | opc!(NAND)
            | opc!(NOR)
            | opc!(NXOR)
            | opc!(SHL)
            | opc!(SHR)
            | opc!(RCL)
            | opc!(RCR)
            | opc!(PUSH)
            | opc!(DUP)
            | opc!(CALL)
            | opc!(RET)
            | opc!(JMP)
            | opc!(JEQ)
            | opc!(JNE)
            | opc!(JGT)
            | opc!(JGE)
            | opc!(JLT)
            | opc!(JLE)
            | opc!(PRINT)
            | opc!(EPRINT)
                if *rfl =>
            {
                src.push(val)
            }
            _ => {}
        }
        match opc {
            #[allow(clippy::manual_range_patterns)]
            opc!(CALL)
            | opc!(RET)
            | opc!(JMP)
            | opc!(JEQ)
            | opc!(JNE)
            | opc!(JGT)
            | opc!(JGE)
            | opc!(JLT)
            | opc!(JLE) => dst.push(reg_index!(pc)),
            _ => {}
        }
        if let opc!(CALL) = opc {
            dst.push(reg_index!(lr));
        }
        if let opc!(RET) = opc {
            dst.push(reg_index!(rr));
            src.push(reg_index!(lr));
        }
        (dst, src)
    }

    pub fn target_ram(&self) -> Vec<(bool, uvm, bool)> {
        match self.opc {
            opc!(LOAD) => vec![(self.rfl, self.val, false)],
            opc!(STORE) => vec![(true, self.reg as uvm, true)],
            opc!(PUSH) => vec![(true, reg_index!(sp), true)],
            opc!(DUP) => vec![(self.rfl, self.val, false), (true, reg_index!(sp), true)],
            opc!(POP) => vec![(true, uvm::saturating_sub(reg_index!(sp), REG_LEN as uvm), true)],
            opc!(DROP) => vec![(true, uvm::saturating_sub(reg_index!(sp), REG_LEN as uvm), true)],
            _ => vec![],
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { rfl, opc, reg, val } = self;
        let reg = Registers::register_name(*reg as uvm);
        let val = if *rfl {
            format!("{:<REG_LEN$}", Registers::register_name(*val as uvm))
        } else {
            format!("{val:0>REG_LEN$X}")
        };

        match opc {
            opc!(NOP) => write!(f, "NOP           "),
            opc!(HALT) => write!(f, "HALT      {val}"),
            opc!(SET) => write!(f, "SET    {} {val}", reg),
            opc!(LOAD) => write!(f, "LOAD   {} {val}", reg),
            opc!(STORE) => write!(f, "STORE  {} {val}", reg),
            opc!(ADD) => write!(f, "ADD    {} {val}", reg),
            opc!(SUB) => write!(f, "SUB    {} {val}", reg),
            opc!(MUL) => write!(f, "MUL    {} {val}", reg),
            opc!(DIV) => write!(f, "DIV    {} {val}", reg),
            opc!(MOD) => write!(f, "MOD    {} {val}", reg),
            opc!(PUSH) => write!(f, "PUSH      {val}"),
            opc!(POP) => write!(f, "POP    {}     ", reg),
            opc!(DROP) => write!(f, "DROP          "),
            opc!(CALL) => write!(f, "CALL      {val}"),
            opc!(RET) => write!(f, "RET       {val}"),
            opc!(JMP) => write!(f, "JMP       {val}"),
            opc!(JEQ) => write!(f, "JEQ    {} {val}", reg),
            opc!(JNE) => write!(f, "JNE    {} {val}", reg),
            _ => panic!("Unexpected opcode 0x{opc:02X}"),
        }?;
        Ok(())
    }
}
