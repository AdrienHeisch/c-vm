use crate::uvm;

#[allow(unused)]
#[derive(Default, Debug)]
pub struct Registers {
    pub pc: uvm,
    pub sp: uvm,
    pub bp: uvm,
    pub rr: uvm,
    pub sr: uvm,
    pub fr: uvm,
    pub r0: uvm,
    pub r1: uvm,
    pub r2: uvm,
    pub r3: uvm,
    pub r4: uvm,
    pub r5: uvm,
    pub r6: uvm,
    pub r7: uvm,
}

impl Registers {
    pub fn get(&self, reg_idx: uvm) -> uvm {
        match reg_idx {
            0 => self.pc,
            1 => self.sp,
            2 => self.bp,
            3 => self.rr,
            4 => self.sr,
            5 => self.fr,
            6 => self.r0,
            7 => self.r1,
            8 => self.r2,
            9 => self.r3,
            10 => self.r4,
            11 => self.r5,
            12 => self.r6,
            13 => self.r7,
            _ => panic!("Invalid register index {reg_idx}"),
        }
    }

    pub fn set(&mut self, reg_idx: uvm, value: uvm) {
        match reg_idx {
            0 => self.pc = value,
            1 => self.sp = value,
            2 => self.bp = value,
            3 => self.rr = value,
            4 => self.sr = value,
            5 => self.fr = value,
            6 => self.r0 = value,
            7 => self.r1 = value,
            8 => self.r2 = value,
            9 => self.r3 = value,
            10 => self.r4 = value,
            11 => self.r5 = value,
            12 => self.r6 = value,
            13 => self.r7 = value,
            _ => panic!("Invalid register index {reg_idx}"),
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
            _ => panic!("Invalid register index {reg_idx}"),
        }
        .to_string()
    }
}

#[macro_export]
macro_rules! reg_index {
    (pc) => {
        0
    };
    (sp) => {
        1
    };
    (bp) => {
        2
    };
    (rr) => {
        3
    };
    (sr) => {
        4
    };
    (fr) => {
        5
    };
    (r0) => {
        6
    };
    (r1) => {
        7
    };
    (r2) => {
        8
    };
    (r3) => {
        9
    };
    (r4) => {
        10
    };
    (r5) => {
        11
    };
    (r6) => {
        12
    };
    (r7) => {
        13
    };
}
