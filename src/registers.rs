use crate::uvm;

#[allow(unused)]
#[derive(Default, Debug)]
pub struct Registers {
    pub pc: uvm,
    pub sp: uvm,
    pub bp: uvm,
    pub lr: uvm,
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
            3 => self.lr,
            4 => self.rr,
            5 => self.sr,
            6 => self.fr,
            7 => self.r0,
            8 => self.r1,
            9 => self.r2,
            10 => self.r3,
            11 => self.r4,
            12 => self.r5,
            13 => self.r6,
            14 => self.r7,
            _ => panic!("Invalid register index {reg_idx}"),
        }
    }

    pub fn set(&mut self, reg_idx: uvm, value: uvm) {
        match reg_idx {
            0 => self.pc = value,
            1 => self.sp = value,
            2 => self.bp = value,
            3 => self.lr = value,
            4 => self.rr = value,
            5 => self.sr = value,
            6 => self.fr = value,
            7 => self.r0 = value,
            8 => self.r1 = value,
            9 => self.r2 = value,
            10 => self.r3 = value,
            11 => self.r4 = value,
            12 => self.r5 = value,
            13 => self.r6 = value,
            14 => self.r7 = value,
            _ => panic!("Invalid register index {reg_idx}"),
        }
    }

    pub fn show(&self) -> Vec<String> {
        (0..15)
            .map(|i| format!("{} {:08X}", Self::register_name(i), self.get(i)))
            .collect::<Vec<_>>()
    }

    pub fn register_name(reg_idx: uvm) -> String {
        match reg_idx {
            0 => "PC",
            1 => "SP",
            2 => "BP",
            3 => "LR",
            4 => "RR",
            5 => "SR",
            6 => "FR",
            7 => "R0",
            8 => "R1",
            9 => "R2",
            10 => "R3",
            11 => "R4",
            12 => "R5",
            13 => "R6",
            14 => "R7",
            _ => panic!("Invalid register index {reg_idx}"),
        }
        .to_string()
    }
}
