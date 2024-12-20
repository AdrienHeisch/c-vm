use crate::{instruction::Instruction, opc, registers::Registers, uvm, REG_LEN};

pub const RAM_LEN: usize = 512;

pub fn run(program: &[Instruction]) {
    let mut vm = VM::new();
    while let Some(instruction) = program.get(vm.regs.pc as usize) {
        vm.push_stderr(format!("{:04X} : ", vm.regs.pc));

        if let Some(exit_code) = vm.execute(*instruction) {
            println!("Program exited with code : {exit_code}");
            break;
        }

        eprint!("{}", vm.stderr());
        print!("{}", vm.stdout());
    }
}

pub struct VM {
    regs: Registers,
    ram: [u8; RAM_LEN],
    stdout: String,
    stderr: String,
}

impl VM {
    pub fn new() -> Self {
        Self {
            regs: Default::default(),
            ram: [0; RAM_LEN],
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    pub fn pc(&self) -> uvm {
        self.regs.pc
    }

    pub fn get_reg(&self, idx: uvm) -> uvm {
        self.regs.get(idx)
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
    }

    pub fn show_regs(&self) -> Vec<String> {
        self.regs.show()
    }

    pub fn show_ram(&self) -> Vec<String> {
        self.ram
            .iter()
            .map(|byte| format!("{:02X}", byte))
            .collect()
    }

    pub fn execute(&mut self, instruction: Instruction) -> Option<uvm> {
        let Instruction { rfl, opc, reg, val } = instruction;
        let pc = self.regs.pc;
        let reg = reg as uvm;

        self.push_stderr(format!("{instruction:?}"));

        if opc == opc!(HALT) {
            self.push_stderr("\n".to_string());
            return Some(halt(self, rfl, val));
        }

        match opc {
            opc!(NOP) => nop(),
            opc!(SET) => set(self, rfl, reg, val),
            opc!(LOAD) => load(self, rfl, reg, val),
            opc!(STORE) => store(self, rfl, reg, val),
            opc!(ADD) => add(self, rfl, reg, val),
            opc!(SUB) => sub(self, rfl, reg, val),
            opc!(MUL) => mul(self, rfl, reg, val),
            opc!(DIV) => div(self, rfl, reg, val),
            opc!(MOD) => modl(self, rfl, reg, val),
            opc!(PUSH) => push(self, rfl, val),
            opc!(POP) => pop(self, reg),
            opc!(DROP) => drop(self),
            opc!(CALL) => call(self, rfl, val),
            opc!(RET) => ret(self, rfl, val),
            opc!(JMP) => jmp(self, rfl, val),
            opc!(JEQ) => jeq(self, rfl, reg, val),
            opc!(JNE) => jne(self, rfl, reg, val),
            _ => panic!("Unexpected opcode 0x{opc:02X}"),
        }

        if self.regs.pc == pc {
            self.regs.pc = pc + 1;
        }

        self.push_stderr("\n".to_string());
        None
    }
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
    vm.push_stderr(format!(" => R_ = {value}"));
}

fn load(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    let addr = if rfl { vm.regs.get(val) } else { val } as usize;
    let mut bytes = vm.ram[addr..addr + REG_LEN].to_vec();
    while bytes.len() < REG_LEN {
        bytes.push(0);
    }
    let value = uvm::from_le_bytes(bytes.try_into().unwrap());
    vm.regs.set(reg, value);
    vm.push_stderr(format!(" => @0x{addr:X} -> {value}"));
}

fn store(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    let addr = vm.regs.get(reg) as usize;
    let value = if rfl { vm.regs.get(val) } else { val };
    let bytes = uvm::to_le_bytes(value);
    vm.ram[addr..addr + REG_LEN].copy_from_slice(&bytes[0..REG_LEN]);
    vm.push_stderr(format!(" => @0x{addr:X} = {value}"));
}

fn binop(vm: &mut VM, rfl: bool, reg: uvm, val: uvm, op: fn(uvm, uvm) -> uvm) {
    let val = if rfl { vm.regs.get(val) } else { val };
    let value = op(vm.regs.get(reg), val);
    vm.regs.set(reg, value);
    vm.push_stderr(format!(" => R_ = {value}"));
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
    vm.push_stderr(format!(" => @0x{sp:X} = {value}"));
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

    vm.push_stderr(format!(" => @0x{sp:X} -> {value}"));
}

fn drop(vm: &mut VM) {
    let sp = vm.regs.sp - REG_LEN as uvm;
    vm.regs.sp = sp;
    let mut bytes = vm.ram[sp as usize..sp as usize + REG_LEN].to_vec();
    while bytes.len() < REG_LEN {
        bytes.push(0);
    }
    let value = uvm::from_le_bytes(bytes.try_into().unwrap());

    vm.push_stderr(format!(" => @0x{sp:X} -> {value}"));
}

fn call(vm: &mut VM, rfl: bool, val: uvm) {
    vm.regs.lr = vm.regs.pc + 1;
    jmp(vm, rfl, val);
}

fn ret(vm: &mut VM, rfl: bool, val: uvm) {
    let value = if rfl { vm.regs.get(val) } else { val };
    vm.regs.rr = value;
    vm.regs.pc = vm.regs.lr;
    vm.push_stderr(format!(" => RR = {value}, JMP {}", vm.regs.lr));
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
    vm.push_stderr(format!(" => {cond}"));
}

fn jeq(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    jcond(vm, rfl, reg, val, uvm::eq);
}

fn jne(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    jcond(vm, rfl, reg, val, uvm::ne);
}
