// rasmx86_64 - src/core/sse2.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::core::comp::*;

use crate::shr::{
    ast::{Instruction, Operand},
    atype::{AType, ToAType},
    num::Number,
    reg::Purpose as RPurpose,
};

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
pub fn gen_movxxd(ins: &Instruction, bits: u8, opcrm: &[u8], opcmr: &[u8]) -> Vec<u8> {
    let finopc = match (ins.dst().unwrap(), ins.src().unwrap()) {
        (Operand::Reg(_), Operand::Reg(_) | Operand::Mem(_)) => opcrm,
        (Operand::Mem(_), Operand::Reg(_)) => opcmr,
        _ => panic!("Invalid combo for movxxs!"),
    };
    gen_ins(ins, finopc, (true, None, None), None, bits, false)
}
pub fn gen_movmskpd(ins: &Instruction, bits: u8, iopc: &[u8]) -> Vec<u8> {
    let mut opc = vec![0x66];
    opc.extend(gen_ins(ins, iopc, (true, None, None), None, bits, false));
    opc
}

pub fn ins_movdq(ins: &Instruction, bits: u8) -> Vec<u8> {
    if let AType::Register(RPurpose::General, _) | AType::Memory(_) | AType::SMemory(_) =
        ins.dst().unwrap().atype()
    {
        let mut opc = vec![0x66];
        opc.extend(gen_ins(
            ins,
            &[0x0F, 0x7E],
            (true, None, None),
            None,
            bits,
            false,
        ));
        opc
    } else {
        gen_ins(ins, &[0x0F, 0x6E], (true, None, None), None, bits, false)
    }
}
pub fn ins_paddx(ins: &Instruction, bits: u8, x: u8) -> Vec<u8> {
    let mut opc = vec![0x66];
    opc.extend(match x {
        1 => gen_ins(ins, &[0x0F, 0xFC], (true, None, None), None, bits, false),
        2 => gen_ins(ins, &[0x0F, 0xFD], (true, None, None), None, bits, false),
        3 => gen_ins(ins, &[0x0F, 0xFE], (true, None, None), None, bits, false),
        4 => gen_ins(ins, &[0x0F, 0xD4], (true, None, None), None, bits, false),
        _ => panic!("ins_paddx: x is not in range 1..=4"),
    });
    opc
}
pub fn ins_paddsx(ins: &Instruction, bits: u8, b: bool) -> Vec<u8> {
    if b {
        gen_ins(
            ins,
            &[0x66, 0x0F, 0xEC],
            (true, None, None),
            None,
            bits,
            false,
        )
    } else {
        gen_ins(
            ins,
            &[0x66, 0x0F, 0xED],
            (true, None, None),
            None,
            bits,
            false,
        )
    }
}

pub fn ins_psubx(ins: &Instruction, bits: u8, x: u8) -> Vec<u8> {
    match x {
        1 => gen_ins(
            ins,
            &[0x66, 0x0F, 0xF8],
            (true, None, None),
            None,
            bits,
            false,
        ),
        2 => gen_ins(
            ins,
            &[0x66, 0x0F, 0xF9],
            (true, None, None),
            None,
            bits,
            false,
        ),
        3 => gen_ins(
            ins,
            &[0x66, 0x0F, 0xFA],
            (true, None, None),
            None,
            bits,
            false,
        ),
        _ => panic!("ins_paddx: x is not in range 1..=3"),
    }
}

pub fn ins_psubsx(ins: &Instruction, bits: u8, b: bool) -> Vec<u8> {
    if b {
        gen_ins(
            ins,
            &[0x66, 0x0F, 0xE8],
            (true, None, None),
            None,
            bits,
            false,
        )
    } else {
        gen_ins(
            ins,
            &[0x66, 0x0F, 0xE9],
            (true, None, None),
            None,
            bits,
            false,
        )
    }
}

pub fn ins_pmulx(ins: &Instruction, bits: u8, lower: bool) -> Vec<u8> {
    if lower {
        gen_ins(
            ins,
            &[0x66, 0x0F, 0xD5],
            (true, None, None),
            None,
            bits,
            false,
        )
    } else {
        gen_ins(
            ins,
            &[0x66, 0x0F, 0xE5],
            (true, None, None),
            None,
            bits,
            false,
        )
    }
}
pub fn ins_pmaddwd(ins: &Instruction, bits: u8) -> Vec<u8> {
    gen_ins(
        ins,
        &[0x66, 0x0F, 0xF5],
        (true, None, None),
        None,
        bits,
        false,
    )
}
pub fn ins_cmp(ins: &Instruction, bits: u8, v: u8) -> Vec<u8> {
    let opc = match v {
        1 => &[0x66, 0x0F, 0x74],
        2 => &[0x66, 0x0F, 0x75],
        3 => &[0x66, 0x0F, 0x76],
        4 => &[0x66, 0x0F, 0x64],
        5 => &[0x66, 0x0F, 0x65],
        6 => &[0x66, 0x0F, 0x66],
        _ => panic!("sse2.rs:ins_cmp - v is not in range of 1..=6"),
    };
    gen_ins(ins, opc, (true, None, None), None, bits, false)
}
pub fn ins_pack(ins: &Instruction, bits: u8, v: u8) -> Vec<u8> {
    let opc = match v {
        1 => &[0x66, 0x0F, 0x67],
        2 => &[0x66, 0x0F, 0x63],
        3 => &[0x66, 0x0F, 0x6B],
        _ => panic!("sse2.rs:ins_pack - v is not in range of 1..=3"),
    };
    gen_ins(ins, opc, (true, None, None), None, bits, false)
}

#[inline]
pub fn ins_unpck_h(ins: &Instruction, bits: u8, opc: &[u8]) -> Vec<u8> {
    gen_ins(ins, opc, (true, None, None), None, bits, false)
}

pub fn ins_unpack(ins: &Instruction, bits: u8, v: u8) -> Vec<u8> {
    let opc = match v {
        1 => &[0x66, 0x0F, 0x60],
        2 => &[0x66, 0x0F, 0x61],
        3 => &[0x66, 0x0F, 0x62],
        4 => &[0x66, 0x0F, 0x68],
        5 => &[0x66, 0x0F, 0x69],
        6 => &[0x66, 0x0F, 0x6A],
        _ => panic!("sse2.rs:ins_unpack - v is not in range of 1..=6"),
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
pub fn ins_shuff(ins: &Instruction, bits: u8, opc: &[u8]) -> Vec<u8> {
    let imm = if let Operand::Imm(i) = ins.oprs[2] {
        Some(vec![Number::split_into_bytes(i)[0]])
    } else {
        None
    };
    gen_ins(ins, opc, (true, None, None), imm, bits, false)
}

pub fn sgen_ins_alt(ins: &Instruction, bits: u8, pref: u8, opc: &[u8]) -> Vec<u8> {
    let mut fin_opc = vec![pref];
    fin_opc.extend(opc);
    let imm = if let Some(Operand::Imm(n)) = ins.oprs.get(2) {
        Some(vec![n.split_into_bytes()[0]])
    } else {
        None
    };
    gen_ins(ins, &fin_opc, (true, None, None), imm, bits, false)
}
