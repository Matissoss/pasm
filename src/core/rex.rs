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
};

fn needs_rex(ins: &Instruction) -> bool{
    if ins.size() != Size::Qword {
        return false;
    }
    match &ins.mnem{
        Mnm::ADD|Mnm::MOV => true,
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
    // does some things in SIB.index, but that is for later
    let x : u8 = 0;
    // will be modified in sometime, because REX.B can also mean SIB.base :)
    let b : u8 = if let Some(Operand::Reg(reg)) = ins.dst(){
        if reg.needs_rex() {1} else {0} 
    } else {0};

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
