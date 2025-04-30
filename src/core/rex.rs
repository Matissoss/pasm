// rasmx86_64 - src/core/rex.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

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
    let (size_d, size_s) = match (ins.dst(), ins.src()){
        (Some(d), Some(s)) => (d.size(), s.size()),
        (Some(d), None) => (d.size(), Size::Unknown),
        (None   , Some(s)) => (Size::Unknown, s.size()),
        _ => (Size::Unknown, Size::Unknown)
    };
    match (size_d, size_s) {
        (Size::Qword, Size::Qword)|(Size::Qword, _)|(_, Size::Qword) => {},
        _ => return false
    }
    match &ins.mnem{
        Mnm::MOV => {
            if let (Some(Operand::Reg(_)), Some(Operand::Reg(_)))|
            (Some(Operand::Mem(_)|Operand::Segment(_)), _)|(_, Some(Operand::Mem(_)|Operand::Segment(_))) = (ins.dst(), ins.src()){
                    return true;
            }
            return false;
        },
        Mnm::SUB|Mnm::ADD|Mnm::IMUL|Mnm::CMP|Mnm::TEST|Mnm::DEC|Mnm::INC|Mnm::OR|Mnm::AND|Mnm::NOT|Mnm::NEG|
        Mnm::XOR
            => {
            if let (_, Some(Operand::Reg(_)))|(Some(Operand::Reg(_)), _)|
            (Some(Operand::Mem(_)|Operand::Segment(_)), _)|(_, Some(Operand::Mem(_)|Operand::Segment(_))) = (ins.dst(), ins.src()){
                    return true;
            }
            return false;
        },
        Mnm::SAR|Mnm::SAL|Mnm::SHL|Mnm::SHR|Mnm::LEA => true,
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

fn calc_rex(ins: &Instruction) -> u8{
    // fixed pattern
    let base = 0b0100_0000;
    let w : u8 = if ins.mnem.defaults_to_64bit() {0} else {1};
    let r : u8 = if let Some(Operand::Reg(reg)) = ins.src(){
        if reg.needs_rex() {1} else {0} 
    } else {0};

    let mut b : u8 = if let Some(Operand::Reg(reg)) = ins.dst(){
        if reg.needs_rex() {1} else {0} 
    } else { 0 };
    
    let mut x : u8 = 0;
    if let (_, Some(Operand::Segment(m)))|(Some(Operand::Segment(m)), _) = (ins.dst(), ins.src()){
        match m.address {
            Mem::SIB(_, i, _, _)|Mem::SIBOffset(_,i,_,_,_)|
            Mem::Index(i, _, _)|Mem::IndexOffset(i,_,_,_) =>{
                if i.needs_rex(){
                    x = 1;
                }
            }
            _ => {}
        }
    }
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
    if let (_, Some(Operand::Segment(m)))|(Some(Operand::Segment(m)), _) = (ins.dst(), ins.src()){
        match m.address {
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
