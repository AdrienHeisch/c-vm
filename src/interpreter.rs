use std::slice::Iter;

#[derive(Debug, Clone, Copy)]
struct Instruction {
    rfl: bool,
    opc: u8,
    reg: u8,
    val: u64,
}

const RAM_LEN: usize = 32;

struct State {
    regs: Registers,
    ram: [u8; RAM_LEN],
}

impl State {
    fn default() -> Self {
        Self {
            regs: Default::default(),
            ram: [0; RAM_LEN],
        }
    }
}

/// PC, SP, BP, RR, SR, FR, PC, R0, R1, R2, R3...
#[allow(unused)]
#[derive(Default, Debug)]
struct Registers ([uvm; 15]);

#[allow(non_camel_case_types)]
type uvm = u64;

impl Registers {
    const REG_LEN: usize = uvm::BITS as usize / 8;

    fn pc(&self) -> usize {
        self.0[0] as usize
    }

    fn pc_value(&self) -> uvm {
        self.0[0]
    }

    fn set_pc(&mut self, value: uvm) {
        self.0[0] = value
    }

    fn sp(&self) -> usize {
        self.0[1] as usize
    }

    fn sp_value(&self) -> uvm {
        self.0[1]
    }

    fn set_sp(&mut self, value: uvm) {
        self.0[1] = value
    }

    fn get(&self, reg_idx: uvm) -> uvm {
        if let Some(reg) = self.0.get(reg_idx as usize) {
            *reg
        } else {
            panic!("Invalid register index {reg_idx}")
        }
    }

    fn set(&mut self, reg_idx: uvm, value: uvm) {
        if let Some(reg) = self.0.get_mut(reg_idx as usize) {
            *reg = value
        } else {
            panic!("Invalid register index {reg_idx}")
        }
    }
}

pub fn interpret(bytes: &[u8]) {
    let program = read(bytes);
    let mut state = State::default();
    while let Some(instruction) = program.get(state.regs.pc()) {
        let pc_before = state.regs.pc();
        print!("{:04} : ", state.regs.pc());
        if let Some(exit_code) = execute(&mut state, *instruction) {
            println!("Program exited with code : {exit_code}");
            break;
        }
        // println!("regs: {:?}", state.regs);
        if state.regs.pc() == pc_before {
            state.regs.set_pc(state.regs.pc_value() + 1);
        }
    }
}

fn read(bytes: &[u8]) -> Vec<Instruction> {
    let mut bytes = bytes.iter();
    let mut program = Vec::new();
    while let Some(instruction) = collect_instruction(&mut bytes) {
        program.push(instruction);
    }
    program
}

fn collect_instruction(bytes: &mut Iter<u8>) -> Option<Instruction> {
    let (rfl, opc) = if let Some(byte) = bytes.next() {
        (byte & 0b10000000 != 0, byte & 0b01111111)
    } else {
        return None;
    };

    let reg = if let Some(byte) = bytes.next() {
        *byte
    } else {
        return None;
    };

    let val = if let Some(byte) = bytes.next() {
        if rfl {
            *byte as u64
        } else {
            // TODO extract function copy from register
            let mut val = Vec::with_capacity(Registers::REG_LEN);
            val.push(*byte);
            for _ in 1..Registers::REG_LEN {
                if let Some(byte) = bytes.next() {
                    val.push(*byte);
                } else {
                    return None;
                };
            }
            while val.len() < 8 {
                val.push(0);
            }
            u64::from_le_bytes(val.try_into().unwrap())
        }
    } else {
        return None;
    };

    let instruction = Instruction { rfl, opc, reg, val };

    Some(instruction)
}

fn execute(state: &mut State, instruction: Instruction) -> Option<u8> {
    let Instruction { rfl, opc, reg, val } = instruction;

    let reg = reg as uvm;
    let regval = state.regs.get(reg);
    let pfx = if rfl { '%' } else { '$' };

    match opc {
        // NOP
        0x00 => {
            println!("NOP");
        }
        // SET
        0x04 => {
            let value = if rfl { state.regs.get(val) } else { val };
            state.regs.set(reg, value);
            println!("SET %{reg} {pfx}{val} => {value}");
        }
        // LOAD
        0x05 => {
            let addr = regval as usize;
            let mut bytes = state.ram[addr..addr + Registers::REG_LEN].to_vec();
            while bytes.len() < 8 {
                bytes.push(0);
            }
            let value = u64::from_le_bytes(bytes.try_into().unwrap());
            state.regs.set(reg, value);

            println!("LOAD %{} {pfx}{} => {} at address {}", reg, val, value, addr);
            println!("RAM: {:?}", state.ram);
        }
        // STORE
        0x06 => {
            let addr = regval as usize;
            let value = if rfl { state.regs.get(val) } else { val };
            let bytes = u64::to_le_bytes(value);
            state.ram[addr..addr + Registers::REG_LEN].copy_from_slice(&bytes[0..Registers::REG_LEN]);

            println!("STORE %{} {pfx}{} => {} at address {}", reg, val, value, addr);
            println!("RAM: {:?}", state.ram);
        }
        // ADD
        0x0C => {
            let val = if rfl { state.regs.get(val) } else { val };
            let value = regval + val;
            state.regs.set(reg, value);

            println!("ADD %{} {pfx}{} => {}", reg, val, value);
        }
        // SUB
        0x0D => {
            let val = if rfl { state.regs.get(val) } else { val };
            let value = regval - val;
            state.regs.set(reg, value);

            println!("SUB %{} {pfx}{} => {}", reg, val, value);
        }
        // PUSH
        0x1C => {
            let value = if rfl { regval } else { val };
            let bytes = u64::to_le_bytes(value);
            state.ram[state.regs.sp()..state.regs.sp() + Registers::REG_LEN].copy_from_slice(&bytes[0..Registers::REG_LEN]);
            state.regs.set_sp(state.regs.sp_value() + Registers::REG_LEN as uvm);

            println!("PUSH %{reg} => {regval} at SP {}", state.regs.sp() - Registers::REG_LEN);
            println!("RAM: {:?}", state.ram);
        }
        // POP
        0x1E if rfl => {
            state.regs.set_sp(state.regs.sp_value() - Registers::REG_LEN as uvm);
            let mut bytes = state.ram[state.regs.sp()..state.regs.sp() + Registers::REG_LEN].to_vec();
            while bytes.len() < 8 {
                bytes.push(0);
            }
            let value = u64::from_le_bytes(bytes.try_into().unwrap());
            state.regs.set(reg, value);

            state.ram[state.regs.sp()..state.regs.sp() + Registers::REG_LEN].copy_from_slice(&([0, 0, 0, 0, 0, 0, 0, 0][0..Registers::REG_LEN]));
            println!("POP %{reg} => {value} at SP {}", state.regs.sp());
            println!("RAM: {:?}", state.ram);
        }
        // JMP
        0x22 => {
            let addr = if rfl { state.regs.get(val) } else { val };
            state.regs.set_pc(addr);
            println!("JMP {pfx}{addr}");
        }
        // JEQ
        0x23 => {
            let addr = if rfl { state.regs.get(val) } else { val };
            let jmp = regval == 0;
            if jmp {
                state.regs.set_pc(addr);
            }
            println!("JEQ %{reg} {pfx}{val} => {}", jmp);
        }
        // JNE
        0x24 => {
            let addr = if rfl { state.regs.get(val) } else { val };
            let jmp = regval != 0;
            if jmp {
                state.regs.set_pc(addr);
            }
            println!("JNE %{reg} {pfx}{val} => {}", jmp);
        }
        _ => panic!("Unexpected opcode {opc:02X}"),
    }

    None
}
