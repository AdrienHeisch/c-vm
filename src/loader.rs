use crate::{instruction::Instruction, uvm, REG_LEN};
use std::slice::Iter;

pub fn decode(bytes: &[u8], address: usize) -> Option<Instruction> {
    let mut bytes = bytes
        .get(address..)
        .unwrap_or_else(|| panic!("Read out of memory at 0x{address:X}"))
        .iter();
    if let Some(instruction) = collect_instruction(&mut bytes) {
        return Some(instruction);
    }
    None
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

    let val = if rfl {
        (*bytes.next()?).into()
    } else {
        // TODO extract function copy from register
        let mut val = [0; REG_LEN];
        for b in &mut val {
            *b = *bytes.next()?;
        }
        uvm::from_le_bytes(val)
    };

    let instruction = Instruction { rfl, opc, reg, val };

    Some(instruction)
}
