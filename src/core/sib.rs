// pasm - src/core/sib.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{ast::Operand, mem::Mem, reg::Register};

pub fn gen_sib_ins(dst: &Option<Operand>) -> Option<u8> {
    if let Some(Operand::Mem(mem)) = dst {
        return gen_sib(mem);
    }
    None
}

pub fn gen_sib(mem: &Mem) -> Option<u8> {
    if mem.is_sib() {
        let base = if let Some(r) = mem.base() {
            r.to_byte()
        } else {
            0b101
        };
        let index = mem.index().unwrap();
        let scale = if mem.scale().is_any() {
            0
        } else {
            mem.scale() as u8
        };
        Some(sib(scale, index.to_byte(), base))
    } else if mem.is_riprel() {
        None
    } else if let Some(Register::RSP | Register::ESP | Register::SP) = mem.base() {
        Some(sib(0, Register::SP.to_byte(), Register::SP.to_byte()))
    } else {
        None
    }
}

#[inline(always)]
const fn sib(scale: u8, index: u8, base: u8) -> u8 {
    (scale << 6) | (index << 3) | base
}
