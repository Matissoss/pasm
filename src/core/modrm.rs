// rasmx86_64 - modrm.rs
// ---------------------
// made by matissoss
// licensed under MPL

use crate::shr::{
    //reg::Register as Reg,
    mem::Mem,
    ast::Operand as Op,
    num::Number
};

type OP = Option<Op>;
pub fn gen_modrm(dst: OP, src: OP, reg: Option<u8>) -> u8{
    let mod_ = match (&dst, &src) {
        (None, Some(Op::Mem(Mem::Direct(_, _))))|(Some(Op::Mem(Mem::Direct(_,_))), None) => 0b00,
        (None, Some(Op::Mem(Mem::Offset(_, o, _))))|(Some(Op::Mem(Mem::Offset(_, o, _))), None) => {
            let n = Number::squeeze_i64(*o as i64);
            match n {
                Number::Int8(_) => 0b01,
                _ => 0b10
            }
        }
        _ => 0b11
    };

    let reg = if let Some(reg) = reg {reg}
    else{
        if let Some(Op::Reg(src)) = src{
            src.to_byte()
        }
        else{
            0
        }   
    };

    let rm  = if let Some(Op::Reg(dst)) = dst{
        dst.to_byte()
    }
    else{
        0
    };

    return (mod_ << 6) + (reg << 3) + rm
}
