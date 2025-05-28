// rasmx86_64 - src/core/sib.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    ast::{Instruction, Operand},
    mem::Mem,
    reg::Register,
};

pub fn gen_sib_ins(ins: &Instruction) -> Option<u8> {
    if let Some(dst) = ins.dst() {
        if let Some(sib) = gen_sib(dst) {
            return Some(sib);
        }
    }
    if let Some(src) = ins.src() {
        if let Some(sib) = gen_sib(src) {
            return Some(sib);
        }
    }
    None
}

pub fn gen_sib(op: &Operand) -> Option<u8> {
    match op {
        Operand::Segment(s) => match s.address {
            Mem::SIB(base, index, scale, _) | Mem::SIBOffset(base, index, scale, _, _) => {
                Some(sib(scale as u8, index.to_byte(), base.to_byte()))
            }
            Mem::Index(index, scale, _) | Mem::IndexOffset(index, _, scale, _) => {
                Some(sib(scale as u8, index.to_byte(), 0b101))
            }
            Mem::Offset(base, _, _) => {
                if base == Register::RSP {
                    Some(sib(0, 0, base.to_byte()))
                } else {
                    None
                }
            }
            _ => None,
        },
        Operand::Mem(Mem::Offset(base, _, _)) => {
            if base == &Register::RSP {
                Some(sib(0, base.to_byte(), base.to_byte()))
            } else {
                None
            }
        }
        Operand::Mem(
            Mem::SIB(base, index, scale, _) | Mem::SIBOffset(base, index, scale, _, _),
        ) => Some(sib(*scale as u8, index.to_byte(), base.to_byte())),
        Operand::Mem(Mem::Index(index, scale, _) | Mem::IndexOffset(index, _, scale, _)) => {
            Some(sib(*scale as u8, index.to_byte(), 0b101))
        }
        _ => None,
    }
}

#[inline(always)]
const fn sib(scale: u8, index: u8, base: u8) -> u8 {
    (scale << 6) | (index << 3) | base
}
