// rasmx86_64 - rex.rs
// -------------------
// made by matissoss
// licensed under MPL

use crate::shr::{
    ast::{
        Instruction,
        Operand,
    },
    size::Size,
    ins::Mnemonic as Mnm,
    mem::Mem
};

fn needs_rex(ins: &Instruction) -> bool{
    if ins.size() != Size::Qword{
        return false;
    }
    match &ins.mnem{
        Mnm::MOV => {
            if let (Some(Operand::Reg(_)), Some(Operand::Reg(_)))|
            (Some(Operand::Mem(_)), _)|(_, Some(Operand::Mem(_))) = (ins.dst(), ins.src()){
                    return true;
            }
            return false;
        },
        Mnm::SUB|Mnm::ADD|Mnm::IMUL => {
            if let (_, Some(Operand::Reg(_)))|(Some(Operand::Reg(_)), _)|
            (Some(Operand::Mem(_)), _)|(_, Some(Operand::Mem(_))) = (ins.dst(), ins.src()){
                    return true;
            }
            return false;
        }
        _        => {
            if let Some(Operand::Reg(dst)) = ins.dst(){
                if dst.needs_rex(){
                    return true;
                }
            }
            if let Some(Operand::Reg(src)) = ins.src(){
                if src.needs_rex(){
                    return true;
                }
            }
            return false;
        },
    }
}

fn defaults_to_64bit(ins: &Mnm) -> bool{
    return match ins {
        Mnm::PUSH|Mnm::POP => true,
        _ => false
    }
}

fn calc_rex(ins: &Instruction) -> u8{
    // fixed pattern
    let base = 0b0100_0000;
    let w : u8 = if defaults_to_64bit(&ins.mnem) {0} else {1};
    let r : u8 = if let Some(Operand::Reg(reg)) = ins.src(){
        if reg.needs_rex() {1} else {0} 
    } else {0};

    let mut x : u8 = 0;
    if let (_, Some(Operand::Mem(m)))|(Some(Operand::Mem(m)), _) = (ins.dst(), ins.src()){
        match m {
            Mem::SIB(_, i, _, _)|Mem::SIBOffset(_,i,_,_,_)|Mem::Index(i, _, _)|Mem::IndexOffset(i,_,_,_) =>{
                if i.needs_rex(){
                    x = 1;
                }
            }
            _ => {}
        }
    }

    let mut b : u8 = if let Some(Operand::Reg(reg)) = ins.dst(){
        if reg.needs_rex() {1} else {0} 
    } else { 0 };

    if let (_, Some(Operand::Mem(m)))|(Some(Operand::Mem(m)), _) = (ins.dst(), ins.src()){
        match m {
            Mem::SIB(base,_,_,_)|Mem::SIBOffset(base,_,_,_,_) =>{
                if base.needs_rex(){
                    b = 1;
                }
            }
            _ => {}
        }
    }

    return base + (w << 3) + (r << 2) + (x << 1) + b;
}

pub fn gen_rex(ins: &Instruction) -> Option<u8>{
    if needs_rex(ins) {
        return Some(calc_rex(ins));
    }
    else {
        return None;
    }
}
