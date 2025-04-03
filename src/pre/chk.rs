// rasmx86_64 - chk.rs
// -------------------
// made by matissoss
// licensed under MPL
// ===================
// Logical syntax check

use crate::color::ColorText;
use crate::shr::{
    ins::Instruction as Ins,
    ast::{
        AST,
        ASTInstruction,
        AsmType,
        ToAsmType
    },
};

#[derive(Debug)]
pub enum LogErr{
    NoDst(Ins, usize),
    NoSrc(Ins, usize),
    UnexpSrc(Ins, usize),
    UnexpDst(Ins, usize),
    WrongSrc(Ins, usize),
    WrongDst(Ins, usize)
}

pub fn check_file(file: &AST) -> Option<Vec<LogErr>>{
    let mut errors : Vec<LogErr> = Vec::new();
    for label in &file.labels{
        for instr in &label.inst{
            if let Some(err) = chk_ins(instr){
                errors.push(err);
            }
        }
    }
    if errors.is_empty() == false{
        Some(errors)
    }
    else {
        None
    }

}

type OVA = Option<Vec<AsmType>>;
fn type_chk(inst: &ASTInstruction, left: OVA, right: OVA) -> Option<LogErr>{
    if let Some(dst_types) = left{
        let dst = inst.dst.clone().expect("assert instruction.dst == Some failed");
        if !dst_types.contains(&dst.asm_type()){
            return Some(LogErr::WrongDst(inst.ins, inst.lin));
        }
    }
    if let Some(src_types) = right{
        let src = inst.src.clone().expect("assert instruction.src == Some failed");
        if !src_types.contains(&src.asm_type()){
            return Some(LogErr::WrongSrc(inst.ins, inst.lin));
        }
    }

    return None;
}

fn inst_has(inst: &ASTInstruction, left: bool, right: bool) -> Option<LogErr>{
    if left {
        if let None = inst.dst{
            return Some(LogErr::NoDst(inst.ins, inst.lin));
        }
        if let Some(_) = inst.src{
            if !right{
                return Some(LogErr::UnexpSrc(inst.ins, inst.lin));
            }
        }
    }
    if right{
        if let None = inst.src {
            return Some(LogErr::NoSrc(inst.ins, inst.lin));
        }
        if let Some(_) = inst.dst{
            if !left{
                return Some(LogErr::UnexpDst(inst.ins, inst.lin));
            }
        }
    }
    return None;
}

fn chk_ins(inst: &ASTInstruction) -> Option<LogErr> {
    match inst.ins {
        Ins::MOV|Ins::ADD |Ins::SUB|Ins::IMUL
       |Ins::MUL|Ins::IDIV|Ins::DIV|Ins::AND
       |Ins::OR |Ins::NOT |Ins::XOR|Ins::SHL
       |Ins::SHR|Ins::TEST
            => {
            if let Some(err) = inst_has(&inst, true, true){
                return Some(err);
            }
            else {
                return type_chk(
                    inst, 
                    Some(vec![AsmType::Reg, AsmType::Mem]),
                    Some(vec![AsmType::Reg, AsmType::Mem, AsmType::Imm, AsmType::ConstRef])
                )
            }
        },
        Ins::JMP|Ins::JE|Ins::JNE|Ins::JG
       |Ins::JGE|Ins::JL|Ins::JLE|Ins::JZ
       |Ins::JNZ|Ins::CALL => {
           if let Some(err) = inst_has(&inst, true, false){
               return Some(err);
           }
           else {
               return type_chk(
                   inst, 
                   Some(vec![AsmType::LabelRef]),
                   None
               );
           }
        },
        Ins::CMP => {
            if let Some(err) = inst_has(&inst, true, true){
                return Some(err)
            }
            else {
                return type_chk(
                    inst,
                    Some(vec![AsmType::Imm, AsmType::Reg, AsmType::Mem, AsmType::ConstRef]),
                    Some(vec![AsmType::Imm, AsmType::Reg, AsmType::Mem, AsmType::ConstRef])
                );
            }
        }
        Ins::INC|Ins::DEC|Ins::POP|Ins::PUSH => {
            if let Some(err) = inst_has(&inst, true, false){
                return Some(err);
            }
            else {
                return type_chk(
                    inst,
                    Some(vec![AsmType::Mem, AsmType::Reg]),
                    None
                );
            }
        },
        Ins::SYSCALL|Ins::RET => return inst_has(&inst, false, false)
    }
}

impl ToString for LogErr{
    fn to_string(&self) -> String {
        match self {
            Self::NoDst(i,l) => {
                format!("{}:\n\tAt Line {}\n\tNo Destination found for instruction: {:?}", 
                    "error".red(), l.to_string().as_str().bold_yellow(), i)
            },
            Self::NoSrc(i,l) => {
                format!("{}:\n\tAt Line {}\n\tNo Source found for instruction: {:?}", 
                    "error".red(), l.to_string().as_str().bold_yellow(), i)
            },
            Self::UnexpSrc(i,l) => {
                format!("{}:\n\tAt Line {}\n\tUnexpected Source found for instruction: {:?}", 
                    "error".red(), l.to_string().as_str().bold_yellow(), i)
            },
            Self::UnexpDst(i,l) => {
                format!("{}:\n\tAt Line {}\n\tUnexpected Destination found for instruction: {:?}", 
                    "error".red(), l.to_string().as_str().bold_yellow(), i)
            },
            Self::WrongDst(i,l) => {
                format!("{}:\n\tAt Line {}\n\tWrong Destination found for instruction: {:?}", 
                    "error".red(), l.to_string().as_str().bold_yellow(), i)

            },
            Self::WrongSrc(i,l) => {
                format!("{}:\n\tAt Line {}\n\tWrong Source found for instruction: {:?}", 
                    "error".red(), l.to_string().as_str().bold_yellow(), i)
            }
        }
    }
}
