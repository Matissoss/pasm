// rasmx86_64 - src/core/sib.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{ast::Instruction, mem::Mem, reg::Register};

pub fn gen_sib_ins(ins: &Instruction) -> Option<u8> {
    if let Some(mem) = ins.get_mem() {
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
        let scale = mem.scale().unwrap();
        Some(sib(scale as u8, index.to_byte(), base))
    } else if mem.is_riprel() {
        None
    } else {
        if let (Some(_), Some(Register::RSP)) = (mem.offset(), mem.base()) {
            Some(sib(0, Register::RSP.to_byte(), Register::RSP.to_byte()))
        } else {
            None
        }
    }
}

#[inline(always)]
const fn sib(scale: u8, index: u8, base: u8) -> u8 {
    (scale << 6) | (index << 3) | base
}
