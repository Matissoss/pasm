// rasmx86_64 - src/core/sib.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{ast::Operand, mem::Mem};

pub fn gen_sib(op: &Operand) -> Option<u8> {
    match op {
        Operand::Segment(s) => match s.address {
            Mem::SIB(base, index, scale, _) | Mem::SIBOffset(base, index, scale, _, _) => {
                Some(sib(scale as u8, index.to_byte(), base.to_byte()))
            }
            Mem::Index(index, scale, _) | Mem::IndexOffset(index, _, scale, _) => {
                Some(sib(scale as u8, index.to_byte(), 0b101))
            }
            _ => None,
        },
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
