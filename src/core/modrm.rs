// rasmx86_64 - modrm.rs
// ---------------------
// made by matissoss
// licensed under MPL

use crate::shr::{
    //reg::Register as Reg,
    //mem::Mem,
    ast::Operand as Op
};

type OP = Option<Op>;
pub fn gen_modrm(dst: OP, src: OP, reg: Option<u8>) -> u8{
    // for now only register-direct mode
    let mod_ = 0b11;

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
