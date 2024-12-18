use crate::uvm;

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub rfl: bool,
    pub opc: u8,
    pub reg: u8,
    pub val: uvm,
}