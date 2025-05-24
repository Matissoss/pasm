// rasmx86_64 - src/core/sse.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::core::comp::*;

use crate::shr::ast::{Instruction, Operand};

pub fn sgen_ins(ins: &Instruction, bits: u8, ps: bool, opc: &[u8]) -> Vec<u8> {
    let mut fin_opc = if !ps { vec![0xF3] } else { vec![] };
    fin_opc.extend(opc);
    let imm = if let Some(Operand::Imm(n)) = ins.oprs.get(2) {
        Some(vec![n.split_into_bytes()[0]])
    } else {
        None
    };
    gen_ins(ins, &fin_opc, (true, None, None), imm, bits, false)
}

pub fn gen_movxxs(ins: &Instruction, bits: u8, opcrm: &[u8], opcmr: &[u8]) -> Vec<u8> {
    let finopc = match (ins.dst().unwrap(), ins.src().unwrap()) {
        (Operand::Reg(_), Operand::Reg(_) | Operand::Mem(_)) => opcrm,
        (Operand::Mem(_), Operand::Reg(_)) => opcmr,
        _ => panic!("Invalid combo for movxxs!"),
    };
    gen_ins(ins, finopc, (true, None, None), None, bits, false)
}

pub fn gen_cvt4x(ins: &Instruction, bits: u8, iopc: &[u8]) -> Vec<u8> {
    let mut opc = vec![0xF3];
    opc.extend(gen_ins(ins, iopc, (true, None, None), None, bits, true));
    opc
}
