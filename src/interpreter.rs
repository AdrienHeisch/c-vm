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

#[allow(unused)]
#[derive(Default, Debug)]
struct Registers {
    pc: usize,
    sp: usize,
    bp: usize,
    rr: usize,
    sr: usize,
    fr: u8,
    pr: u64,
    c0: u8,
    c1: u8,
    s0: u16,
    s1: u16,
    i0: u32,
    i1: u32,
    l0: u64,
    l1: u64,
    f0: f32,
    f1: f32,
    d0: f64,
    d1: f64,
    p0: usize,
    p1: usize,
}

impl Registers {
    fn len(reg_idx: u8) -> usize {
        match reg_idx {
            0 => 8,
            1 => 8,
            2 => 8,
            3 => 8,
            4 => 8,
            5 => 1,
            6 => 8,
            7 => 1,
            8 => 1,
            9 => 2,
            10 => 2,
            11 => 4,
            12 => 4,
            13 => 8,
            14 => 8,
            15 => 4,
            16 => 4,
            17 => 8,
            18 => 8,
            19 => 8,
            20 => 8,
            _ => panic!("Invalid register index {reg_idx}"),
        }
    }

    fn get(&self, reg_idx: u8) -> u64 {
        match reg_idx {
            0 => self.pc as u64,
            1 => self.sp as u64,
            2 => self.bp as u64,
            3 => self.rr as u64,
            4 => self.sr as u64,
            5 => self.fr as u64,
            6 => self.pr,
            7 => self.c0 as u64,
            8 => self.c1 as u64,
            9 => self.s0 as u64,
            10 => self.s1 as u64,
            11 => self.i0 as u64,
            12 => self.i1 as u64,
            13 => self.l0,
            14 => self.l1,
            15 => self.f0 as u64,
            16 => self.f1 as u64,
            17 => self.d0 as u64,
            18 => self.d1 as u64,
            19 => self.p0 as u64,
            20 => self.p1 as u64,
            _ => panic!("Invalid register index {reg_idx}"),
        }
    }

    fn set(&mut self, reg_idx: u8, value: u64) {
        match reg_idx {
            0 => self.pc = value as usize,
            1 => self.sp = value as usize,
            2 => self.bp = value as usize,
            3 => self.rr = value as usize,
            4 => self.sr = value as usize,
            5 => self.fr = value as u8,
            6 => self.pr = value,
            7 => self.c0 = value as u8,
            8 => self.c1 = value as u8,
            9 => self.s0 = value as u16,
            10 => self.s1 = value as u16,
            11 => self.i0 = value as u32,
            12 => self.i1 = value as u32,
            13 => self.l0 = value,
            14 => self.l1 = value,
            15 => self.f0 = value as f32,
            16 => self.f1 = value as f32,
            17 => self.d0 = value as f64,
            18 => self.d1 = value as f64,
            19 => self.p0 = value as usize,
            20 => self.p1 = value as usize,
            _ => panic!("Invalid register index {reg_idx}"),
        }
    }
}

pub fn interpret(bytes: &[u8]) {
    let program = read(bytes);
    let mut state = State::default();
    while let Some(instruction) = program.get(state.regs.pc) {
        let pc_before = state.regs.pc;
        print!("{:04} : ", state.regs.pc);
        if let Some(exit_code) = execute(&mut state, *instruction) {
            println!("Program exited with code : {exit_code}");
            break;
        }
        // println!("regs: {:?}", state.regs);
        if state.regs.pc == pc_before {
            state.regs.pc += 1;
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
            let reg_len = Registers::len(reg);
            let mut val = Vec::with_capacity(reg_len);
            val.push(*byte);
            for _ in 1..reg_len {
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

    let regval = state.regs.get(reg);
    let pfx = if rfl { '%' } else { '$' };

    match opc {
        // NOP
        0x00 => {
            println!("NOP");
        }
        // SET
        0x04 => {
            let value = if rfl { state.regs.get(val as u8) } else { val };
            state.regs.set(reg, value);
            println!("SET %{reg} {pfx}{val} => {value}");
        }
        // LOAD
        0x05 => {
            let addr = regval as usize;
            let len = Registers::len(reg);
            let mut bytes = state.ram[addr..addr + len].to_vec();
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
            let value = if rfl { state.regs.get(val as u8) } else { val };
            let bytes = u64::to_le_bytes(value);
            let reg_len = Registers::len(reg);
            state.ram[addr..addr + reg_len].copy_from_slice(&bytes[0..reg_len]);

            println!("STORE %{} {pfx}{} => {} at address {}", reg, val, value, addr);
            println!("RAM: {:?}", state.ram);
        }
        // ADD
        0x0C => {
            let val = if rfl { state.regs.get(val as u8) } else { val };
            let value = regval + val;
            state.regs.set(reg, value);

            println!("ADD %{} {pfx}{} => {}", reg, val, value);
        }
        // SUB
        0x0D => {
            let val = if rfl { state.regs.get(val as u8) } else { val };
            let value = regval - val;
            state.regs.set(reg, value);

            println!("SUB %{} {pfx}{} => {}", reg, val, value);
        }
        // PUSH
        0x1C => {
            let value = if rfl { regval } else { val };
            let bytes = u64::to_le_bytes(value);
            let reg_len = Registers::len(reg);
            state.ram[state.regs.sp..state.regs.sp + reg_len].copy_from_slice(&bytes[0..reg_len]);
            state.regs.sp += reg_len;

            println!("PUSH %{reg} => {regval} at SP {}", state.regs.sp - reg_len);
            println!("RAM: {:?}", state.ram);
        }
        // POP
        0x1E if rfl => {
            let reg_len = Registers::len(reg);
            state.regs.sp -= reg_len;
            let mut bytes = state.ram[state.regs.sp..state.regs.sp + reg_len].to_vec();
            while bytes.len() < 8 {
                bytes.push(0);
            }
            let value = u64::from_le_bytes(bytes.try_into().unwrap());
            state.regs.set(reg, value);

            state.ram[state.regs.sp..state.regs.sp + reg_len].copy_from_slice(&([0, 0, 0, 0, 0, 0, 0, 0][0..reg_len]));
            println!("POP %{reg} => {value} at SP {}", state.regs.sp);
            println!("RAM: {:?}", state.ram);
        }
        // JMP
        0x22 => {
            let addr = if rfl { state.regs.get(val as u8) } else { val } as usize;
            state.regs.pc = addr;
            println!("JMP {pfx}{addr}");
        }
        // JEQ
        0x23 => {
            let addr = if rfl { state.regs.get(val as u8) } else { val } as usize;
            let jmp = regval == 0;
            if jmp {
                state.regs.pc = addr;
            }
            println!("JEQ %{reg} {pfx}{val} => {}", jmp);
        }
        // JNE
        0x24 => {
            let addr = if rfl { state.regs.get(val as u8) } else { val } as usize;
            let jmp = regval != 0;
            if jmp {
                state.regs.pc = addr;
            }
            println!("JNE %{reg} {pfx}{val} => {}", jmp);
        }
        _ => panic!("Unexpected opcode {opc:02X}"),
    }

    None
}
