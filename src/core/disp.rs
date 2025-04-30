// rasmx86_64 - src/core/disp.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    ast::Operand as Op,
    mem::Mem,
    num::Number,
};

pub fn gen_disp(op: &Op) -> Option<Vec<u8>>{
    match op {
        Op::Segment(s) => {
            match s.address{
                Mem::Offset(_, o, _)|Mem::SIBOffset(_,_,_,o,_)|Mem::IndexOffset(_,o,_,_)=> {
                    if (o as i8) as i32 == o{
                        return Some(Number::Int8(o as i8).split_into_bytes());
                    }
                    return Some(Number::Int32(o).split_into_bytes());
                },
                Mem::Index(_,_,_) => {
                    return Some(vec![0;4]);
                }
                _ => None,
            }
        }
        Op::Mem(Mem::Offset(_, o, _)|Mem::SIBOffset(_,_,_,o,_)|Mem::IndexOffset(_,o,_,_))=> {
            // using type casting ;)
            if (*o as i8) as i32 == *o{
                return Some(Number::Int8(*o as i8).split_into_bytes());
            }
            return Some(Number::Int32(*o).split_into_bytes());
        },
        Op::Mem(Mem::Index(_,_,_)) => {
            return Some(vec![0;4]);
        }
        _ => None,
    }
}
