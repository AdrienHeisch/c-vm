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
    loop {
        let pc = state.regs.get_pc();
        if let Some(instruction) = program.get(pc as usize) {
            print!("{:04} : ", pc);
            if let Some(exit_code) = execute(&mut state, *instruction) {
                println!("Program exited with code : {exit_code}");
                break;
            }
            // println!("regs: {:?}", state.regs);
            let new_pc = state.regs.get_pc();
            if new_pc == pc {
                state.regs.set_pc(new_pc + 1);
            }
        } else {
            break;
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
        0x1C => push(state, rfl, val),
        0x1E => pop(state, reg),
        0x1F => drop(state),
        0x20 => call(state, rfl, val),
        0x21 => ret(state, rfl, val),
        0x22 => jmp(state, rfl, val),
        0x23 => jeq(state, rfl, reg, val),
        0x24 => jne(state, rfl, reg, val),
        _ => panic!("Unexpected opcode 0x{opc:02X}"),
    }

    println!();

    None
}

fn nop() {}

fn halt() -> uvm {
    todo!();
}

fn set(state: &mut State, rfl: bool, reg: uvm, val: uvm) {
    let value = if rfl { state.regs.get(val) } else { val };
    state.regs.set(reg, value);
    print!(" => R_ = {value}");
}

fn load(state: &mut State, reg: uvm) {
    let addr = state.regs.get(reg) as usize;
    let mut bytes = state.ram[addr..addr + REG_LEN].to_vec();
    while bytes.len() < REG_LEN {
        bytes.push(0);
    }
    let value = uvm::from_le_bytes(bytes.try_into().unwrap());
    state.regs.set(reg, value);
    print!(" => @0x{addr:X} = {value}");
}

fn store(state: &mut State, rfl: bool, reg: uvm, val: uvm) {
    let addr = state.regs.get(reg) as usize;
    let value = if rfl { state.regs.get(val) } else { val };
    let bytes = uvm::to_le_bytes(value);
    state.ram[addr..addr + REG_LEN].copy_from_slice(&bytes[0..REG_LEN]);
    print!(" => @0x{addr:X} = {value}");
}

fn add(state: &mut State, rfl: bool, reg: uvm, val: uvm) {
    let val = if rfl { state.regs.get(val) } else { val };
    let value = state.regs.get(reg) + val;
    state.regs.set(reg, value);
    print!(" => R_ = {value}");
}

fn sub(state: &mut State, rfl: bool, reg: uvm, val: uvm) {
    let val = if rfl { state.regs.get(val) } else { val };
    let value = state.regs.get(reg) - val;
    state.regs.set(reg, value);
    print!(" => R_ = {value}");
}

fn push(state: &mut State, rfl: bool, val: uvm) {
    let value = if rfl { state.regs.get(val) } else { val };
    let bytes = uvm::to_le_bytes(value);
    let sp = state.regs.get_sp();
    state.ram[sp as usize..sp as usize + REG_LEN].copy_from_slice(&bytes[0..REG_LEN]);
    state.regs.set_sp(sp + REG_LEN as uvm);
    print!(" => @0x{sp:X} = {value}");
}

fn pop(state: &mut State, reg: uvm) {
    let sp = state.regs.get_sp() - REG_LEN as uvm;
    state.regs.set_sp(sp);
    let mut bytes = state.ram[sp as usize..sp as usize + REG_LEN].to_vec();
    while bytes.len() < REG_LEN {
        bytes.push(0);
    }
    let value = uvm::from_le_bytes(bytes.try_into().unwrap());
    state.regs.set(reg, value);

    print!(" => @0x{sp:X} -> {value}");
}

fn drop(state: &mut State) {
    let sp = state.regs.get_sp() - REG_LEN as uvm;
    state.regs.set_sp(sp);
    let mut bytes = state.ram[sp as usize..sp as usize + REG_LEN].to_vec();
    while bytes.len() < REG_LEN {
        bytes.push(0);
    }
    let value = uvm::from_le_bytes(bytes.try_into().unwrap());
    
    print!(" => @0x{sp:X} -> {value}");
}

fn call(state: &mut State, rfl: bool, val: uvm) {
    push(state, rfl, state.regs.get_bp());
    push(state, rfl, state.regs.get_pc() + 1);
    jmp(state, rfl, val);
}

fn ret(state: &mut State, _: bool, _: uvm) {
    // POP R0,
    // POP BP,
    // JMP (Reg R0)
    pop(state, state.regs.r0() as uvm);
    pop(state, state.regs.bp() as uvm);
    jmp(state, true, state.regs.r0() as uvm);
    print!(" => JMP {}", state.regs.get(state.regs.r0() as uvm));
}

fn jmp(state: &mut State, rfl: bool, val: uvm) {
    let addr = if rfl { state.regs.get(val) } else { val };
    state.regs.set_pc(addr);
}

fn jcond(state: &mut State, rfl: bool, reg: uvm, val: uvm, op: fn(&uvm, &uvm) -> bool) {
    let cond = op(&state.regs.get(reg), &0);
    if cond {
        let addr = if rfl { state.regs.get(val) } else { val };
        state.regs.set_pc(addr);
    }
    print!(" => {cond}");
}

fn jeq(state: &mut State, rfl: bool, reg: uvm, val: uvm) {
    jcond(state, rfl, reg, val, uvm::eq);
}

fn jne(state: &mut State, rfl: bool, reg: uvm, val: uvm) {
    jcond(state, rfl, reg, val, uvm::ne);
}
