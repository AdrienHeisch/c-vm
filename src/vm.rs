use crate::{instruction::Instruction, registers::Registers, uvm, REG_LEN};

pub const RAM_LEN: usize = 128;

pub struct VM {
    regs: Registers,
    ram: [u8; RAM_LEN],
    stdout: String,
    stderr: String,
}

#[allow(dead_code)]
impl VM {
    pub fn new() -> Self {
        Self {
            regs: Default::default(),
            ram: [0; RAM_LEN],
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    fn push_stdout(&mut self, string: String) {
        for char in string.chars() {
            self.stdout.push(char);
        }
    }

    pub fn stdout(&mut self) -> String {
        let mut str = String::with_capacity(self.stdout.len());
        std::mem::swap(&mut self.stdout, &mut str);
        self.stdout.clear();
        str
    }

    fn push_stderr(&mut self, string: String) {
        for char in string.chars() {
            self.stderr.push(char);
        }
    }

    pub fn stderr(&mut self) -> String {
        let mut str = String::with_capacity(self.stderr.len());
        std::mem::swap(&mut self.stderr, &mut str);
        self.stderr.clear();
        str
        String::from_utf8(bytes).unwrap()
    }

    pub fn show_ram(&self) -> String {
        self.ram
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<String>>()
            .join(" ")
    }
}

pub fn run(program: &[Instruction]) {
    let mut vm = VM::new();
    while let Some(instruction) = program.get(vm.regs.pc as usize) {
        vm.push_stdout(format!("{:04} : ", vm.regs.pc));

        if let Some(exit_code) = execute(&mut vm, *instruction) {
            println!("Program exited with code : {exit_code}");
            break;
        }

        eprint!("{}", vm.stderr());
        print!("{}", vm.stdout());
    }
}

pub fn execute(vm: &mut VM, instruction: Instruction) -> Option<uvm> {
    let Instruction { rfl, opc, reg, val } = instruction;

    let pc = vm.regs.pc;
    let reg = reg as uvm;

    vm.push_stdout(format!("{instruction:?}"));

    if opc == 0x01 {
        vm.push_stdout("\n".to_string());
        return Some(halt(vm, rfl, val));
    }

    match opc {
        0x00 => nop(),
        0x04 => set(vm, rfl, reg, val),
        0x05 => load(vm, rfl, reg, val),
        0x06 => store(vm, rfl, reg, val),
        0x0C => add(vm, rfl, reg, val),
        0x0D => sub(vm, rfl, reg, val),
        0x0E => mul(vm, rfl, reg, val),
        0x0F => div(vm, rfl, reg, val),
        0x10 => modl(vm, rfl, reg, val),
        0x1D => push(vm, rfl, val),
        0x1F => pop(vm, reg),
        0x20 => drop(vm),
        0x21 => call(vm, rfl, val),
        0x22 => ret(vm, rfl, val),
        0x23 => jmp(vm, rfl, val),
        0x24 => jeq(vm, rfl, reg, val),
        0x25 => jne(vm, rfl, reg, val),
        _ => panic!("Unexpected opcode 0x{opc:02X}"),
    }

    if vm.regs.pc == pc {
        vm.regs.pc = pc + 1;
    }

    vm.push_stdout("\n".to_string());
    None
}

fn nop() {}

fn halt(vm: &mut VM, rfl: bool, val: uvm) -> uvm {
    if rfl {
        vm.regs.get(val)
    } else {
        val
    }
}

fn set(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    let value = if rfl { vm.regs.get(val) } else { val };
    vm.regs.set(reg, value);
    vm.push_stdout(format!(" => R_ = {value}"));
}

fn load(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    let addr = if rfl { vm.regs.get(val) } else { val } as usize;
    let mut bytes = vm.ram[addr..addr + REG_LEN].to_vec();
    while bytes.len() < REG_LEN {
        bytes.push(0);
    }
    let value = uvm::from_le_bytes(bytes.try_into().unwrap());
    vm.regs.set(reg, value);
    vm.push_stdout(format!(" => @0x{addr:X} -> {value}"));
}

fn store(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    let addr = vm.regs.get(reg) as usize;
    let value = if rfl { vm.regs.get(val) } else { val };
    let bytes = uvm::to_le_bytes(value);
    vm.ram[addr..addr + REG_LEN].copy_from_slice(&bytes[0..REG_LEN]);
    vm.push_stdout(format!(" => @0x{addr:X} = {value}"));
}

fn binop(vm: &mut VM, rfl: bool, reg: uvm, val: uvm, op: fn(uvm, uvm) -> uvm) {
    let val = if rfl { vm.regs.get(val) } else { val };
    let value = op(vm.regs.get(reg), val);
    vm.regs.set(reg, value);
    vm.push_stdout(format!(" => R_ = {value}"));
}

fn add(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    binop(vm, rfl, reg, val, |a, b| a + b);
}

fn sub(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    binop(vm, rfl, reg, val, |a, b| a - b);
}

fn mul(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    binop(vm, rfl, reg, val, |a, b| a * b);
}

fn div(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    binop(vm, rfl, reg, val, |a, b| a / b);
}

fn modl(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    binop(vm, rfl, reg, val, |a, b| a % b);
}

fn push(vm: &mut VM, rfl: bool, val: uvm) {
    let value = if rfl { vm.regs.get(val) } else { val };
    let bytes = uvm::to_le_bytes(value);
    let sp = vm.regs.sp;
    vm.ram[sp as usize..sp as usize + REG_LEN].copy_from_slice(&bytes[0..REG_LEN]);
    vm.regs.sp = sp + REG_LEN as uvm;
    vm.push_stdout(format!(" => @0x{sp:X} = {value}"));
}

fn pop(vm: &mut VM, reg: uvm) {
    let sp = vm.regs.sp - REG_LEN as uvm;
    vm.regs.sp = sp;
    let mut bytes = vm.ram[sp as usize..sp as usize + REG_LEN].to_vec();
    while bytes.len() < REG_LEN {
        bytes.push(0);
    }
    let value = uvm::from_le_bytes(bytes.try_into().unwrap());
    vm.regs.set(reg, value);

    vm.push_stdout(format!(" => @0x{sp:X} -> {value}"));
}

fn drop(vm: &mut VM) {
    let sp = vm.regs.sp - REG_LEN as uvm;
    vm.regs.sp = sp;
    let mut bytes = vm.ram[sp as usize..sp as usize + REG_LEN].to_vec();
    while bytes.len() < REG_LEN {
        bytes.push(0);
    }
    let value = uvm::from_le_bytes(bytes.try_into().unwrap());

    vm.push_stdout(format!(" => @0x{sp:X} -> {value}"));
}

fn call(vm: &mut VM, rfl: bool, val: uvm) {
    vm.regs.lr = vm.regs.pc + 1;
    jmp(vm, rfl, val);
}

fn ret(vm: &mut VM, rfl: bool, val: uvm) {
    let value = if rfl { vm.regs.get(val) } else { val };
    vm.regs.rr = value;
    if vm.regs.lr == 2 {
        panic!("LR: {}", vm.regs.lr);
    }
    jmp(vm, false, vm.regs.lr);
    vm.push_stdout(format!(" => JMP {}", vm.regs.lr));
}

fn jmp(vm: &mut VM, rfl: bool, val: uvm) {
    let addr = if rfl { vm.regs.get(val) } else { val };
    vm.regs.pc = addr;
}

fn jcond(vm: &mut VM, rfl: bool, reg: uvm, val: uvm, op: fn(&uvm, &uvm) -> bool) {
    let cond = op(&vm.regs.get(reg), &0);
    if cond {
        let addr = if rfl { vm.regs.get(val) } else { val };
        vm.regs.pc = addr;
    }
    vm.push_stdout(format!(" => {cond}"));
}

fn jeq(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    jcond(vm, rfl, reg, val, uvm::eq);
}

fn jne(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    jcond(vm, rfl, reg, val, uvm::ne);
}
