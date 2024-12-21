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
    (lr) => {
        6
    };
    (r0) => {
        7
    };
    (r1) => {
        8
    };
    (r2) => {
        9
    };
    (r3) => {
        10
    };
    (r4) => {
        11
    };
    (r5) => {
        12
    };
    (r6) => {
        13
    };
    (r7) => {
        14
    };
}

#[macro_export]
macro_rules! opc {
    (NOP) => {
        0x00
    };
    (HALT) => {
        0x01
    };
    (SYCALL) => {
        0x02
    };
    (CLEAR) => {
        0x03
    };
    (SET) => {
        0x04
    };
    (LOAD) => {
        0x05
    };
    (STORE) => {
        0x06
    };
    (SWAP) => {
        0x07
    };
    (CMP) => {
        0x08
    };
    (NEG) => {
        0x09
    };
    (INC) => {
        0x0A
    };
    (DEC) => {
        0x0B
    };
    (ADD) => {
        0x0C
    };
    (SUB) => {
        0x0D
    };
    (MUL) => {
        0x0E
    };
    (DIV) => {
        0x0F
    };
    (MOD) => {
        0x10
    };
    (NOT) => {
        0x11
    };
    (AND) => {
        0x12
    };
    (OR) => {
        0x13
    };
    (XOR) => {
        0x14
    };
    (NAND) => {
        0x15
    };
    (NOR) => {
        0x16
    };
    (NXOR) => {
        0x17
    };
    (SHL) => {
        0x18
    };
    (SHR) => {
        0x19
    };
    (RCL) => {
        0x1A
    };
    (RCR) => {
        0x1B
    };
    (BSWAP) => {
        0x1C
    };
    (PUSH) => {
        0x1D
    };
    (DUP) => {
        0x1E
    };
    (POP) => {
        0x1F
    };
    (DROP) => {
        0x20
    };
    (CALL) => {
        0x21
    };
    (RET) => {
        0x22
    };
    (JMP) => {
        0x23
    };
    (JEQ) => {
        0x24
    };
    (JNE) => {
        0x25
    };
    (JGT) => {
        0x26
    };
    (JGE) => {
        0x27
    };
    (JLT) => {
        0x28
    };
    (JLE) => {
        0x29
    };
    (PRINT) => {
        0x2A
    };
    (EPRINT) => {
        0x2B
    };
    (DUMP) => {
        0x2C
    };
}
