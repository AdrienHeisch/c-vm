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
            6 => "R0",
            7 => "R1",
            8 => "R2",
            9 => "R3",
            10 => "R4",
            11 => "R5",
            12 => "R6",
            13 => "R7",
            _ => panic!("Invalid register index {reg_idx}")
        }.to_string()
    }
}