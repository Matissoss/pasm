// rasmx86_64 - src/core/sib.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    ast::Operand,
    mem::Mem,
};

pub fn gen_sib(op: &Operand) -> Option<u8>{
    match op{
        Operand::Mem(Mem::RipRelative(_, _)) => {
                // why? because it means 32-bit displacement (no base, no index)
            return Some(25);
        }
        Operand::Mem(Mem::SIB(base, index, scale, _)|Mem::SIBOffset(base, index, scale, _, _)) => {
            let scale_b : u8 = (*scale as u8) << 6;
            let index_b : u8 = index.to_byte() << 3;
            let base_b  : u8 = base.to_byte();
            return Some(scale_b + index_b + base_b);
        },
        Operand::Mem(Mem::Index(index,scale,_)|Mem::IndexOffset(index,_,scale,_)) => {
            let scale_b : u8 = (*scale as u8) << 6;
            let index_b : u8 = index.to_byte() << 3;
            let base_b  : u8 = 0b101;
            return Some(scale_b + index_b + base_b);
        }
        _ => None,
    }
}
