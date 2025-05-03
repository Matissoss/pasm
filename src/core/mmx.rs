// rasmx86_64 - src/core/mmx.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::core::comp::*;
use crate::shr::atype::{AType, ToAType};

use crate::shr::{
    ast::{Instruction, Operand},
    //mem::Mem,
    //num::Number,
    reg::{/*Register,*/ Purpose as RPurpose},
};

pub fn ins_movdq(ins: &Instruction, bits: u8) -> Vec<u8> {
    if let AType::Register(RPurpose::General, _) | AType::Memory(_) | AType::SMemory(_) =
        ins.dst().unwrap().atype()
    {
        gen_ins(ins, &[0x0F, 0x7E], (true, None, None), None, bits, false)
    } else {
        gen_ins(ins, &[0x0F, 0x6E], (true, None, None), None, bits, false)
    }
}
pub fn ins_paddx(ins: &Instruction, bits: u8, x: u8) -> Vec<u8> {
    match x {
        1 => gen_ins(ins, &[0x0F, 0xFC], (true, None, None), None, bits, false),
        2 => gen_ins(ins, &[0x0F, 0xFD], (true, None, None), None, bits, false),
        3 => gen_ins(ins, &[0x0F, 0xFE], (true, None, None), None, bits, false),
        4 => gen_ins(ins, &[0x0F, 0xD4], (true, None, None), None, bits, false),
        _ => panic!("ins_paddx: x is not in range 1..=4"),
    }
}
pub fn ins_paddsx(ins: &Instruction, bits: u8, b: bool) -> Vec<u8> {
    if b {
        gen_ins(ins, &[0x0F, 0xEC], (true, None, None), None, bits, false)
    } else {
        gen_ins(ins, &[0x0F, 0xED], (true, None, None), None, bits, false)
    }
}

pub fn ins_psubx(ins: &Instruction, bits: u8, x: u8) -> Vec<u8> {
    match x {
        1 => gen_ins(ins, &[0x0F, 0xF8], (true, None, None), None, bits, false),
        2 => gen_ins(ins, &[0x0F, 0xF9], (true, None, None), None, bits, false),
        3 => gen_ins(ins, &[0x0F, 0xFA], (true, None, None), None, bits, false),
        _ => panic!("ins_paddx: x is not in range 1..=3"),
    }
}

pub fn ins_psubsx(ins: &Instruction, bits: u8, b: bool) -> Vec<u8> {
    if b {
        gen_ins(ins, &[0x0F, 0xE8], (true, None, None), None, bits, false)
    } else {
        gen_ins(ins, &[0x0F, 0xE9], (true, None, None), None, bits, false)
    }
}

pub fn ins_pmulx(ins: &Instruction, bits: u8, lower: bool) -> Vec<u8> {
    if lower {
        gen_ins(ins, &[0x0F, 0xD5], (true, None, None), None, bits, false)
    } else {
        gen_ins(ins, &[0x0F, 0xE5], (true, None, None), None, bits, false)
    }
}
pub fn ins_pmaddwd(ins: &Instruction, bits: u8) -> Vec<u8> {
    gen_ins(ins, &[0x0F, 0xF5], (true, None, None), None, bits, false)
}
pub fn ins_cmp(ins: &Instruction, bits: u8, v: u8) -> Vec<u8> {
    let opc = match v {
        1 => &[0x0F, 0x74],
        2 => &[0x0F, 0x75],
        3 => &[0x0F, 0x76],
        4 => &[0x0F, 0x64],
        5 => &[0x0F, 0x65],
        6 => &[0x0F, 0x66],
        _ => panic!("mmx.rs:ins_cmp - v is not in range of 1..=6"),
    };
    gen_ins(ins, opc, (true, None, None), None, bits, false)
}
pub fn ins_pack(ins: &Instruction, bits: u8, v: u8) -> Vec<u8> {
    let opc = match v {
        1 => &[0x0F, 0x67],
        2 => &[0x0F, 0x63],
        3 => &[0x0F, 0x6B],
        _ => panic!("mmx.rs:ins_pack - v is not in range of 1..=3"),
    };
    gen_ins(ins, opc, (true, None, None), None, bits, false)
}

pub fn ins_unpack(ins: &Instruction, bits: u8, v: u8) -> Vec<u8> {
    let opc = match v {
        1 => &[0x0F, 0x60],
        2 => &[0x0F, 0x61],
        3 => &[0x0F, 0x62],
        4 => &[0x0F, 0x68],
        5 => &[0x0F, 0x69],
        6 => &[0x0F, 0x6A],
        _ => panic!("mmx.rs:ins_unpack - v is not in range of 1..=6"),
    };
    gen_ins(ins, opc, (true, None, None), None, bits, false)
}

pub fn ins_shift(ins: &Instruction, bits: u8, opc: &[u8], opc_imm: &[u8], modrm: u8) -> Vec<u8> {
    if let Operand::Imm(i) = ins.src().unwrap() {
        gen_ins(
            ins,
            opc_imm,
            (true, Some(modrm), None),
            Some(vec![i.split_into_bytes()[0]]),
            bits,
            false,
        )
    } else {
        gen_ins(ins, opc, (true, None, None), None, bits, false)
    }
}
