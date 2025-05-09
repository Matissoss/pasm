// rasmx86_64 - src/core/sse4.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::core::comp::*;

use crate::shr::ast::{Instruction, Operand};

pub fn sgen_ins(ins: &Instruction, bits: u8, pd: bool, opc: &[u8]) -> Vec<u8> {
    let imm = if let Some(Operand::Imm(n)) = ins.oprs.get(2) {
        Some(vec![n.split_into_bytes()[0]])
    } else {
        None
    };
    let mut fin_opc = Vec::new();
    let genr_ins = gen_ins(ins, opc, (true, None, None), imm, bits, true);
    if !genr_ins.starts_with(&[0x66]) {
        fin_opc.push(if pd { 0x66 } else { 0xF2 });
    }
    fin_opc.extend(genr_ins);
    fin_opc
}
pub fn sgen_ins_alt(ins: &Instruction, bits: u8, p: u8, opc: &[u8]) -> Vec<u8> {
    let imm = if let Some(Operand::Imm(n)) = ins.oprs.get(2) {
        Some(vec![n.split_into_bytes()[0]])
    } else {
        None
    };
    let mut fin_opc = Vec::new();
    let genr_ins = gen_ins(ins, opc, (true, None, None), imm, bits, true);
    if !genr_ins.starts_with(&[0x66]) {
        fin_opc.push(p);
        fin_opc.extend(genr_ins);
    } else {
        fin_opc.push(0x66);
        fin_opc.push(p);
        fin_opc.extend(&genr_ins[1..]);
    }
    fin_opc
}
