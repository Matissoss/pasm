// rasmx86_64 - src/core/disp.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    ast::{Instruction, Operand as Op},
    reg::Register,
    segment::Segment,
};

pub fn gen_disp_ins(ins: &Instruction) -> Option<Vec<u8>> {
    if let Some(dst) = ins.dst() {
        if let Some(disp) = gen_disp(dst) {
            return Some(disp);
        }
    }
    if let Some(src) = ins.src() {
        if let Some(disp) = gen_disp(src) {
            return Some(disp);
        }
    }
    None
}

pub fn gen_disp(op: &Op) -> Option<Vec<u8>> {
    match op {
        Op::Mem(m)
        | Op::Segment(Segment {
            segment: _,
            address: m,
        }) => {
            if let Some((offs, sz)) = m.offset_x86() {
                if sz == 1 {
                    Some(vec![offs[0]])
                } else {
                    Some(offs.to_vec())
                }
            } else {
                if let (Some(Register::RBP | Register::BP | Register::EBP), Some(_)) =
                    (m.base(), m.index())
                {
                    Some(vec![0; 4])
                } else {
                    None
                }
            }
        }
        _ => None,
    }
}
