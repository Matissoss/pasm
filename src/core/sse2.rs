// rasmx86_64 - src/core/sse2.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::core::comp::*;

use crate::shr::ast::{Instruction, Operand};

pub fn sgen_ins(ins: &Instruction, bits: u8, pd: bool, opc: &[u8]) -> Vec<u8> {
    let mut fin_opc = if pd { vec![0x66] } else { vec![0xF2] };
    fin_opc.extend(opc);
    let imm = if let Some(Operand::Imm(n)) = ins.oprs.get(2) {
        Some(vec![n.split_into_bytes()[0]])
    } else {
        None
    };
    gen_ins(ins, &fin_opc, (true, None, None), imm, bits, false)
}
