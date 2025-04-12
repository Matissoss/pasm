// rasmx86_64 - disp.rs
// --------------------
// made by matissoss
// licensed under MPL

use crate::shr::{
    ast::Operand as Op,
    mem::Mem,
    num::Number
};

pub fn gen_disp(op: Op) -> Option<Vec<u8>>{
    match op {
        Op::Mem(Mem::MemAddrWOffset(_, o, _)) => {
            // using type casting ;)
            if (o as i8) as i32 == o{
                return Some(Number::Int8(o as i8).split_into_bytes());
            }
            return Some(Number::Int32(o).split_into_bytes());
        },
        _ => None,
    }
}
