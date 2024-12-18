use crate::uvm;

/// PC, SP, BP, RR, SR, FR, R0, R1, R2, R3...
#[allow(unused)]
#[derive(Default, Debug)]
pub struct Registers ([uvm; 16]);

impl Registers {
    pub fn pc(&self) -> usize {
        self.0[0] as usize
    }

    pub fn pc_value(&self) -> uvm {
        self.0[0]
    }

    pub fn set_pc(&mut self, value: uvm) {
        self.0[0] = value
    }

    pub fn sp(&self) -> usize {
        self.0[1] as usize
    }

    pub fn sp_value(&self) -> uvm {
        self.0[1]
    }

    pub fn set_sp(&mut self, value: uvm) {
        self.0[1] = value
    }

    pub fn get(&self, reg_idx: uvm) -> uvm {
        if let Some(reg) = self.0.get(reg_idx as usize) {
            *reg
        } else {
            panic!("Invalid register index {reg_idx}")
        }
    }

    pub fn set(&mut self, reg_idx: uvm, value: uvm) {
        if let Some(reg) = self.0.get_mut(reg_idx as usize) {
            *reg = value
        } else {
            panic!("Invalid register index {reg_idx}")
        }
    }

    pub fn register_name(reg_idx: uvm) -> String {
        match reg_idx {
            0 => "PC",
            1 => "SP",
            2 => "BP",
            3 => "RR",
            4 => "SR",
            5 => "FR",
            6 => "LR",
            7 => "R0",
            8 => "R1",
            9 => "R2",
            10 => "R3",
            11 => "R4",
            12 => "R5",
            13 => "R6",
            14 => "R7",
            _ => panic!("Invalid register index {reg_idx}")
        }.to_string()
    }
}