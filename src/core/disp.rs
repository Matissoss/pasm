// pasm - src/core/disp.rs
// -----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{ast::Operand, mem::Mem, reg::Register};

pub fn gen_disp_ins(dst: &Option<Operand>, bits: u8) -> Option<Vec<u8>> {
    if let Some(Operand::Mem(mem)) = dst {
        return gen_disp(mem, bits);
    }
    None
}

pub fn gen_disp(mem: &Mem, bits: u8) -> Option<Vec<u8>> {
    if let Some((offs, sz)) = mem.offset_x86() {
        if sz == 1 && !mem.is_riprel() {
            Some(vec![offs[0]])
        } else if bits == 16 {
            Some(offs[0..2].to_vec())
        } else {
            Some(offs.to_vec())
        }
    } else if let (Some(Register::RBP | Register::BP | Register::EBP), Some(_)) =
        (mem.base(), mem.index())
    {
        Some(vec![0; if bits != 16 { 4 } else { 2 }])
    } else {
        None
    }
}
