use crate::{instruction::Instruction, loader, opc, registers::Registers, uvm, REG_LEN};

pub const RAM_LEN: usize = 1024;

const REOM: &str = "READ OUT OF MEMORY";
const WEOM: &str = "WRITE OUT OF MEMORY";

pub fn run(program: &[u8]) {
    let mut vm = VM::new();
    vm.load(program);
    while let Some(instruction) = vm.decode() {
        vm.push_stderr(&format!("{:04X} : ", vm.regs.pc));

        if let Some(exit_code) = vm.execute(instruction) {
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
        let mut ram = [0u8; RAM_LEN];
        ram.iter_mut().for_each(|b| *b = rand::random());
        Self {
            regs: Registers::default(),
            ram,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    pub fn load(&mut self, program: &[u8]) -> uvm {
        for (idx, byte) in program.iter().enumerate() {
            *self.ram.get_mut(idx).expect(WEOM) = *byte;
        }
        let end = program.len() as uvm;
        self.regs.sp = end;
        self.regs.bp = end;
        end
    }

    pub fn pc(&self) -> uvm {
        self.regs.pc
    }

    pub fn get_reg(&self, idx: uvm) -> uvm {
        self.regs.get(idx)
    }

    fn push_stdout(&mut self, string: &str) {
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

    fn push_stderr(&mut self, string: &str) {
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
        self.ram.iter().map(|byte| format!("{byte:02X}")).collect()
    }

    pub fn show_program(&self) -> Vec<(Instruction, usize)> {
        let mut program = Vec::new();
        let mut addr = 0;
        while let Some(instruction) = loader::decode(&self.ram, addr) {
            program.push((instruction, addr));
            addr += instruction.len();
        }
        program
    }

    pub fn decode(&self) -> Option<Instruction> {
        loader::decode(&self.ram, self.regs.pc as usize)
    }

    pub fn execute(&mut self, instruction: Instruction) -> Option<uvm> {
        let Instruction { rfl, opc, reg, val } = instruction;
        let pc = self.regs.pc;
        let reg = reg.into();

        self.push_stderr(&format!("{instruction:?}"));

        if opc == opc!(HALT) {
            self.push_stderr("\n");
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
            opc!(AND) => and(self, rfl, reg, val),
            opc!(PUSH) => push(self, rfl, val),
            opc!(POP) => pop(self, reg),
            opc!(DROP) => drop(self),
            opc!(CALL) => call(self, rfl, val),
            opc!(RET) => ret(self, rfl, val),
            opc!(JMP) => jmp(self, rfl, val),
            opc!(JEQ) => jeq(self, rfl, reg, val),
            opc!(JNE) => jne(self, rfl, reg, val),
            opc!(PRINT) => stdout(self, rfl, val),
            _ => panic!("Unexpected opcode 0x{opc:02X}"),
        }

        if self.regs.pc == pc {
            self.regs.pc = pc + instruction.len() as uvm;
        }

        self.push_stderr("\n");
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

    vm.push_stderr(&format!(" => R_ = {value}"));
}

fn load(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    let addr = if rfl { vm.regs.get(val) } else { val } as usize;
    let bytes = vm
        .ram
        .get(addr..addr + REG_LEN)
        .expect(REOM)
        .try_into()
        .expect(REOM);
    let value = uvm::from_le_bytes(bytes);
    vm.regs.set(reg, value);

    vm.push_stderr(&format!(" => @0x{addr:X} -> {value}"));
}

fn store(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    let addr = vm.regs.get(reg) as usize;
    let value = if rfl { vm.regs.get(val) } else { val };
    let bytes = uvm::to_le_bytes(value);
    vm.ram
        .get_mut(addr..addr + REG_LEN)
        .expect(WEOM)
        .copy_from_slice(&bytes[..]);

    vm.push_stderr(&format!(" => @0x{addr:X} = {value}"));
}

fn binop(vm: &mut VM, rfl: bool, reg: uvm, val: uvm, op: fn(uvm, uvm) -> uvm) {
    let val = if rfl { vm.regs.get(val) } else { val };
    let value = op(vm.regs.get(reg), val);
    vm.regs.set(reg, value);

    vm.push_stderr(&format!(" => R_ = {value}"));
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

fn and(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    binop(vm, rfl, reg, val, |a, b| a & b);
}

fn push(vm: &mut VM, rfl: bool, val: uvm) {
    let value = if rfl { vm.regs.get(val) } else { val };
    let bytes = uvm::to_le_bytes(value);
    let sp = vm.regs.sp;
    vm.ram
        .get_mut(sp as usize..sp as usize + REG_LEN)
        .expect(WEOM)
        .copy_from_slice(&bytes[0..REG_LEN]);
    vm.regs.sp = sp + REG_LEN as uvm;

    vm.push_stderr(&format!(" => @0x{sp:X} = {value}"));
}

fn pop(vm: &mut VM, reg: uvm) {
    let sp = vm.regs.sp - REG_LEN as uvm;
    vm.regs.sp = sp;
    let range = (sp as usize)..(sp as usize) + REG_LEN;
    let bytes = vm.ram.get(range).expect(REOM).try_into().expect(REOM);
    let value = uvm::from_le_bytes(bytes);
    vm.regs.set(reg, value);

    vm.push_stderr(&format!(" => @0x{sp:X} -> {value}"));
}

fn drop(vm: &mut VM) {
    let sp = vm.regs.sp - REG_LEN as uvm;
    vm.regs.sp = sp;
    let bytes = vm
        .ram
        .get((sp as usize)..(sp as usize) + REG_LEN)
        .expect(REOM)
        .try_into()
        .expect(REOM);
    let value = uvm::from_le_bytes(bytes);

    vm.push_stderr(&format!(" => @0x{sp:X} -> {value}"));
}

fn call(vm: &mut VM, rfl: bool, val: uvm) {
    vm.regs.lr = vm.regs.pc + if rfl { 3 } else { 10 };
    jmp(vm, rfl, val);
}

fn ret(vm: &mut VM, rfl: bool, val: uvm) {
    let value = if rfl { vm.regs.get(val) } else { val };
    vm.regs.rr = value;
    vm.regs.pc = vm.regs.lr;

    vm.push_stderr(&format!(" => RR = {value}, JMP {}", vm.regs.lr));
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

    vm.push_stderr(&format!(" => {cond}"));
}

fn jeq(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    jcond(vm, rfl, reg, val, uvm::eq);
}

fn jne(vm: &mut VM, rfl: bool, reg: uvm, val: uvm) {
    jcond(vm, rfl, reg, val, uvm::ne);
}

fn stdout(vm: &mut VM, rfl: bool, val: uvm) {
    let value = if rfl { vm.regs.get(val) } else { val };
    let chars = value.to_le_bytes();
    // println!("{rfl}, {val}");
    // println!("{chars:02X?}");
    let str = String::from_utf8(chars.to_vec()).expect("Invalid string !");
    vm.push_stdout(&str);

    vm.push_stderr(&format!(" => \"{str}\""));
}
