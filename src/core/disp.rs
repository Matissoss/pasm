// pasm - src/core/disp.rs
// -----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{ast::Operand, mem::Mem, reg::Register};

pub fn gen_disp_ins(dst: &Option<Operand>) -> Option<Vec<u8>> {
    if let Some(Operand::Mem(mem)) = dst {
        return gen_disp(mem);
    }
    None
}

pub fn gen_disp(mem: &Mem) -> Option<Vec<u8>> {
    if let Some((offs, sz)) = mem.offset_x86() {
        if sz == 1 && !mem.is_riprel() {
            Some(vec![offs[0]])
        } else {
            Some(offs.to_vec())
        }
    } else if let (Some(Register::RBP | Register::BP | Register::EBP), Some(_)) =
        (mem.base(), mem.index())
    {
        Some(vec![0; 4])
    } else {
        None
    }
}
