// rasmx86_64 - modrm.rs
// ---------------------
// made by matissoss
// licensed under MPL

use crate::shr::{
    //reg::Register as Reg,
    mem::Mem,
    ast::{
        Instruction as Ins,
        Operand as Op
    },
    num::Number
};

pub fn gen_modrm(ins: &Ins, reg: Option<u8>, rm: Option<u8>) -> u8{
    let mod_ : u8 = {
        match (ins.dst(), ins.src()){
            (Some( &Op::Mem(Mem::SIB(_,_,_,_))), _) |
            (_, Some( &Op::Mem(Mem::SIB(_,_,_,_)))) |
            (Some( &Op::Mem(Mem::Direct(_,_))) , _)  |
            (Some( &Op::Mem(Mem::Index(_,_,_))), _) |
            (_, Some( &Op::Mem(Mem::Index(_,_,_)))) |
            (_, Some( &Op::Mem(Mem::Direct(_,_)) )) => 0b00,

            (Some( &Op::Mem(Mem::SIBOffset(_,_,_,o,_)|Mem::Offset(_,o,_))), _)|
            (Some( &Op::Mem(Mem::IndexOffset(_,o,_,_))), _) | (_, Some(&Op::Mem(Mem::IndexOffset(_,o,_,_))))|
            (_, Some( &Op::Mem(Mem::SIBOffset(_,_,_,o,_)|Mem::Offset(_,o,_)))) => {
                match Number::squeeze_i64(o as i64){
                    Number::Int8(_) => 0b01,
                    _ => 0b10,
                }
            },
            _ => 0b11,
        }
    };

    let reg = if let Some(reg) = reg {reg}
    else{
        if let Some(Op::Reg(src)) = ins.src(){
            src.to_byte()
        }
        else{
            0
        }   
    };
    
    let rm = {
        if ins.uses_sib() {0b100}
        else{
            if let Some(rm) = rm {rm}
            else{
                if let Some(Op::Reg(dst)) = ins.dst(){
                    dst.to_byte()
                }
                else{
                    0
                }
            }
        }
    };

    return (mod_ << 6) + (reg << 3) + rm;
}
