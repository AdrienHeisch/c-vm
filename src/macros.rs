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
    (STOREB) => {
        0x06
    };
    (STOREH) => {
        0x07
    };
    (STOREW) => {
        0x08
    };
    (STORED) => {
        0x09
    };
    (SWAP) => {
        0x0A
    };
    (CMP) => {
        0x0B
    };
    (NEG) => {
        0x0C
    };
    (INC) => {
        0x0D
    };
    (DEC) => {
        0x0E
    };
    (ADD) => {
        0x0F
    };
    (SUB) => {
        0x10
    };
    (MUL) => {
        0x11
    };
    (DIV) => {
        0x12
    };
    (MOD) => {
        0x13
    };
    (NOT) => {
        0x14
    };
    (AND) => {
        0x15
    };
    (OR) => {
        0x16
    };
    (XOR) => {
        0x17
    };
    (NAND) => {
        0x18
    };
    (NOR) => {
        0x19
    };
    (NXOR) => {
        0x1A
    };
    (SHL) => {
        0x1B
    };
    (SHR) => {
        0x1C
    };
    (RCL) => {
        0x1D
    };
    (RCR) => {
        0x1E
    };
    (BSWAP) => {
        0x1F
    };
    (PUSH) => {
        0x20
    };
    (DUP) => {
        0x21
    };
    (POP) => {
        0x22
    };
    (DROP) => {
        0x23
    };
    (CALL) => {
        0x24
    };
    (RET) => {
        0x25
    };
    (JMP) => {
        0x26
    };
    (JEQ) => {
        0x27
    };
    (JNE) => {
        0x28
    };
    (JGT) => {
        0x29
    };
    (JGE) => {
        0x2A
    };
    (JLT) => {
        0x2B
    };
    (JLE) => {
        0x2C
    };
    (PRINT) => {
        0x2D
    };
    (EPRINT) => {
        0x2E
    };
    (DUMP) => {
        0x2F
    };
}
