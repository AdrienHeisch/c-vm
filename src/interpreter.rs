use crate::{instruction::Instruction, registers::Registers, uvm, REG_LEN};

const RAM_LEN: usize = 32;

struct State {
    regs: Registers,
    ram: [u8; RAM_LEN],
}

#[allow(dead_code)]
impl State {
    fn print_ram(&self) {
        let hex_string: String = self
            .ram
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<String>>()
            .join(" ");
        println!("RAM: {}", hex_string)
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            regs: Default::default(),
            ram: [0; RAM_LEN],
        }
    }
}

pub fn interpret(program: &[Instruction]) {
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

fn execute(state: &mut State, instruction: Instruction) -> Option<uvm> {
    let Instruction { rfl, opc, reg, val } = instruction;

    let reg = reg as uvm;

    print!("{instruction:?}");

    if opc == 0x01 {
        return Some(halt());
    }

    match opc {
        0x00 => nop(),
        0x04 => set(state, rfl, reg, val),
        0x05 => load(state, reg),
        0x06 => store(state, rfl, reg, val),
        0x0C => add(state, rfl, reg, val),
        0x0D => sub(state, rfl, reg, val),
        0x1C => push(state, rfl, reg, val),
        0x1E => pop(state, reg),
        0x1F => drop(state),
        0x22 => jmp(state, rfl, val),
        0x23 => jeq(state, rfl, reg, val),
        0x24 => jne(state, rfl, reg, val),
        _ => panic!("Unexpected opcode {opc:02X}"),
    }

    None
}

fn nop() {
    println!();
}

fn halt() -> uvm {
    todo!();
}

fn set(state: &mut State, rfl: bool, reg: uvm, val: uvm) {
    let value = if rfl { state.regs.get(val) } else { val };
    state.regs.set(reg, value);
    println!(" => {value}");
}

fn load(state: &mut State, reg: uvm) {
    let addr = state.regs.get(reg) as usize;
    let mut bytes = state.ram[addr..addr + REG_LEN].to_vec();
    while bytes.len() < REG_LEN {
        bytes.push(0);
    }
    let value = uvm::from_le_bytes(bytes.try_into().unwrap());
    state.regs.set(reg, value);
    println!(" => {} @ {}", value, addr);
}

fn store(state: &mut State, rfl: bool, reg: uvm, val: uvm) {
    let addr = state.regs.get(reg) as usize;
    let value = if rfl { state.regs.get(val) } else { val };
    let bytes = uvm::to_le_bytes(value);
    state.ram[addr..addr + REG_LEN].copy_from_slice(&bytes[0..REG_LEN]);
    println!(" => {} @ {}", value, addr);
}

fn add(state: &mut State, rfl: bool, reg: uvm, val: uvm) {
    let val = if rfl { state.regs.get(val) } else { val };
    let value = state.regs.get(reg) + val;
    state.regs.set(reg, value);
    println!(" => {}", value);
}

fn sub(state: &mut State, rfl: bool, reg: uvm, val: uvm) {
    let val = if rfl { state.regs.get(val) } else { val };
    let value = state.regs.get(reg) - val;
    state.regs.set(reg, value);
    println!(" => {}", value);
}

fn push(state: &mut State, rfl: bool, reg: uvm, val: uvm) {
    let value = if rfl { state.regs.get(reg) } else { val };
    let bytes = uvm::to_le_bytes(value);
    state.ram[state.regs.sp()..state.regs.sp() + REG_LEN].copy_from_slice(&bytes[0..REG_LEN]);
    state.regs.set_sp(state.regs.sp_value() + REG_LEN as uvm);
    println!(
        " => {} @ {}",
        state.regs.sp() - REG_LEN,
        state.regs.get(reg)
    );
}

fn pop(state: &mut State, reg: uvm) {
    state.regs.set_sp(state.regs.sp_value() - REG_LEN as uvm);
    let mut bytes = state.ram[state.regs.sp()..state.regs.sp() + REG_LEN].to_vec();
    while bytes.len() < REG_LEN {
        bytes.push(0);
    }
    let value = uvm::from_le_bytes(bytes.try_into().unwrap());
    state.regs.set(reg, value);

    println!(" => {value} @ {}", state.regs.sp());
}

fn drop(state: &mut State) {
    state.regs.set_sp(state.regs.sp_value() - REG_LEN as uvm);
    let mut bytes = state.ram[state.regs.sp()..state.regs.sp() + REG_LEN].to_vec();
    while bytes.len() < REG_LEN {
        bytes.push(0);
    }
    let value = uvm::from_le_bytes(bytes.try_into().unwrap());
    println!(" => {value} @ {}", state.regs.sp());
}

fn jmp(state: &mut State, rfl: bool, val: uvm) {
    let addr = if rfl { state.regs.get(val) } else { val };
    state.regs.set_pc(addr);
    println!();
}

fn jcond(state: &mut State, rfl: bool, reg: uvm, val: uvm, op: fn(&uvm, &uvm) -> bool) {
    let cond = op(&state.regs.get(reg), &0);
    if cond {
        let addr = if rfl { state.regs.get(val) } else { val };
        state.regs.set_pc(addr);
    }
    println!(" => {}", cond);
}

fn jeq(state: &mut State, rfl: bool, reg: uvm, val: uvm) {
    jcond(state, rfl, reg, val, uvm::eq);
}

fn jne(state: &mut State, rfl: bool, reg: uvm, val: uvm) {
    jcond(state, rfl, reg, val, uvm::ne);
}
