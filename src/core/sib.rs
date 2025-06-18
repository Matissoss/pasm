// rasmx86_64 - src/core/sib.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    ast::{Instruction, Operand},
    reg::Register,
    segment::Segment,
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
    if let Some(src) = ins.src2() {
        if let Some(sib) = gen_sib(src) {
            return Some(sib);
        }
    }
    None
}

pub fn gen_sib(op: &Operand) -> Option<u8> {
    match op {
        Operand::Mem(m)
        | Operand::Segment(Segment {
            segment: _,
            address: m,
        }) => {
            if m.is_sib() {
                let base = if let Some(r) = m.base() {
                    r.to_byte()
                } else {
                    0b101
                };
                let index = m.index().unwrap();
                let scale = m.scale().unwrap();
                Some(sib(scale as u8, index.to_byte(), base))
            } else if m.is_riprel() {
                None
            } else {
                if let (Some(_), Some(Register::RSP)) = (m.offset(), m.base()) {
                    Some(sib(0, Register::RSP.to_byte(), Register::RSP.to_byte()))
                } else {
                    None
                }
            }
        }
        _ => None,
    }
}

#[inline(always)]
const fn sib(scale: u8, index: u8, base: u8) -> u8 {
    (scale << 6) | (index << 3) | base
}
