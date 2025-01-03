use crate::{opc, reg_index, registers::Registers, uvm, REG_LEN};
use std::fmt::Debug;

#[derive(Clone, Copy)]
pub struct Instruction {
    pub rfl: bool,
    pub opc: u8,
    pub reg: u8, // TODO uvm ?
    pub val: uvm,
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::manual_range_patterns)]
impl Instruction {
    pub fn len(&self) -> usize {
        if self.rfl {
            3
        } else {
            2 + REG_LEN
        }
    }

    pub fn target_regs(&self) -> (Vec<usize>, Vec<usize>) {
        let Self { rfl, opc, reg, val } = self;
        let (reg, val) = (*reg as usize, *val as usize);
        let (mut dst, mut src) = (Vec::new(), Vec::new());
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
            opc!(STOREB)
            | opc!(STOREH)
            | opc!(STOREW)
            | opc!(STORED)
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
            | opc!(STOREB)
            | opc!(STOREH)
            | opc!(STOREW)
            | opc!(STORED)
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
                src.push(val);
            }
            _ => {}
        }
        match opc {
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
            opc!(STOREB) | opc!(STOREH) | opc!(STOREW) | opc!(STORED) => {
                vec![(true, self.reg.into(), true)]
            }
            opc!(PUSH) => vec![(true, reg_index!(sp), true)],
            opc!(DUP) => vec![(true, reg_index!(sp), true), (self.rfl, self.val, false)],
            opc!(POP) | opc!(DROP) => vec![(
                true,
                uvm::saturating_sub(reg_index!(sp), REG_LEN as uvm),
                true,
            )],
            _ => vec![],
        }
    }

    pub fn target_addr(&self) -> Option<(bool, uvm)> {
        match self.opc {
            opc!(CALL)
            | opc!(RET)
            | opc!(JMP)
            | opc!(JEQ)
            | opc!(JNE)
            | opc!(JGT)
            | opc!(JGE)
            | opc!(JLT)
            | opc!(JLE) => Some((self.rfl, self.val)),
            _ => None,
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { rfl, opc, reg, val } = self;
        let reg = Registers::register_name((*reg).into());
        let val = if *rfl {
            format!("{:<REG_LEN$}", Registers::register_name(*val as uvm))
        } else {
            format!("{val:0>REG_LEN$X}")
        };

        match opc {
            opc!(NOP) => write!(f, "NOP           "),
            opc!(HALT) => write!(f, "HALT      {val}"),
            opc!(SET) => write!(f, "SET    {reg} {val}"),
            opc!(LOAD) => write!(f, "LOAD   {reg} {val}"),
            opc!(STOREB) => write!(f, "STOREB  {reg} {val}"),
            opc!(STOREH) => write!(f, "STOREH  {reg} {val}"),
            opc!(STOREW) => write!(f, "STOREW  {reg} {val}"),
            opc!(STORED) => write!(f, "STORED  {reg} {val}"),
            opc!(ADD) => write!(f, "ADD    {reg} {val}"),
            opc!(SUB) => write!(f, "SUB    {reg} {val}"),
            opc!(MUL) => write!(f, "MUL    {reg} {val}"),
            opc!(DIV) => write!(f, "DIV    {reg} {val}"),
            opc!(MOD) => write!(f, "MOD    {reg} {val}"),
            opc!(AND) => write!(f, "AND    {reg} {val}"),
            opc!(PUSH) => write!(f, "PUSH      {val}"),
            opc!(POP) => write!(f, "POP    {reg}         "),
            opc!(DROP) => write!(f, "DROP              "),
            opc!(CALL) => write!(f, "CALL      {val}"),
            opc!(RET) => write!(f, "RET       {val}"),
            opc!(JMP) => write!(f, "JMP       {val}"),
            opc!(JEQ) => write!(f, "JEQ    {reg} {val}"),
            opc!(JNE) => write!(f, "JNE    {reg} {val}"),
            opc!(PRINT) => write!(f, "PRINT     {val}"),
            _ => write!(f, "INVALID"),
        }?;
        Ok(())
    }
}
