use crate::{uvm, REG_LEN, instruction::Instruction};
use registers::Registers;

mod registers;

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
            let mut bytes = state.ram[addr..addr + REG_LEN].to_vec();
            while bytes.len() < 8 {
                bytes.push(0);
            }
            let value = u64::from_le_bytes(bytes.try_into().unwrap());
            state.regs.set(reg, value);

            println!(
                "LOAD %{} {pfx}{} => {} at address {}",
                reg, val, value, addr
            );
            println!("RAM: {:?}", state.ram);
        }
        // STORE
        0x06 => {
            let addr = regval as usize;
            let value = if rfl { state.regs.get(val) } else { val };
            let bytes = u64::to_le_bytes(value);
            state.ram[addr..addr + REG_LEN]
                .copy_from_slice(&bytes[0..REG_LEN]);

            println!(
                "STORE %{} {pfx}{} => {} at address {}",
                reg, val, value, addr
            );
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
            state.ram[state.regs.sp()..state.regs.sp() + REG_LEN]
                .copy_from_slice(&bytes[0..REG_LEN]);
            state
                .regs
                .set_sp(state.regs.sp_value() + REG_LEN as uvm);

            println!(
                "PUSH %{reg} => {regval} at SP {}",
                state.regs.sp() - REG_LEN
            );
            println!("RAM: {:?}", state.ram);
        }
        // POP
        0x1E if rfl => {
            state
                .regs
                .set_sp(state.regs.sp_value() - REG_LEN as uvm);
            let mut bytes =
                state.ram[state.regs.sp()..state.regs.sp() + REG_LEN].to_vec();
            while bytes.len() < 8 {
                bytes.push(0);
            }
            let value = u64::from_le_bytes(bytes.try_into().unwrap());
            state.regs.set(reg, value);

            state.ram[state.regs.sp()..state.regs.sp() + REG_LEN]
                .copy_from_slice(&([0, 0, 0, 0, 0, 0, 0, 0][0..REG_LEN]));
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
