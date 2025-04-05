// rasmx86_64 - chk.rs
// -------------------
// made by matissoss
// licensed under MPL

use crate::shr::{
    ins::Instruction as Ins,
    ast::{
        AST,
        ASTInstruction,
        AsmType,
        ToAsmType,
        Operand
    },
    error::RASMError,
    error::ExceptionType as ExType
};

pub fn check_file(file: &AST) -> Option<Vec<RASMError>>{
    let mut errors : Vec<RASMError> = Vec::new();
    for label in &file.labels{
        for instr in &label.inst{
            if let Some(err) = chk_ins(instr){
                errors.push(err);
            }
            if let Some(err) = size_chk(instr){
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

fn size_chk(inst: &ASTInstruction) -> Option<RASMError>{
    if let (Some(d), Some(s)) = (&inst.dst, &inst.src){
        if let Operand::Imm(s_i) = s{
            if d.size_bytes() < s_i.size_bytes(){
                return Some(RASMError::new(
                    Some(inst.lin),
                    ExType::Error,
                    Some(inst.to_string()),
                    Some(format!("Destination has smaller size than immediate you tried to assign for destination!")),
                    Some(format!("Make source (immediate) fit in range of {} byte number", d.size_bytes()))
                ));
            }
            return None;
        }
        if d.size_bytes() != s.size_bytes() && !inst.ins.allows_diff_size(){
            return Some(RASMError::new(
                Some(inst.lin),
                ExType::Error,
                Some(inst.to_string()),
                Some(format!("Found 2 different size for operands and instruction doesn't allow for it: {}B for dst, {}B for src",
                    d.size_bytes(), s.size_bytes())),
                Some(format!("Try using same size operand"))
            ));
        }
    }
    return None;
}

type OVA = Option<Vec<AsmType>>;
fn type_chk(inst: &ASTInstruction, left: OVA, right: OVA) -> Option<RASMError>{
    if let Some(dst_types) = left{
        let dst = inst.dst.clone().expect("assert instruction.dst == Some failed");
        if !dst_types.contains(&dst.asm_type()){
            return Some(RASMError::new(
                    Some(inst.lin),
                    ExType::Error,
                    Some(inst.to_string()),
                    Some(format!("Instruction {:?} doesn't have support for: {:?}", inst.ins, dst)),
                    Some(format!("expected = {:?}", dst_types)),
            ));
        }
    }
    if let Some(src_types) = right{
        let src = inst.src.clone().expect("assert instruction.src == Some failed");
        if !src_types.contains(&src.asm_type()){
            return Some(RASMError::new(
                    Some(inst.lin),
                    ExType::Error,
                    Some(format!("{:?} {:?} {:?}", inst, inst.dst, inst.src)),
                    Some(format!("Instruction {:?} doesn't have support for: {:?}", inst.ins, src)),
                    Some(format!("expected = {:?}", src_types)),
            ));
        }
    }

    return None;
}

fn inst_has(inst: &ASTInstruction, left: bool, right: bool) -> Option<RASMError>{
    if left {
        if let None = inst.dst{
            return Some(RASMError::new(
                Some(inst.lin),
                ExType::Error,
                Some(inst.to_string()),
                Some(format!("Expected destination (1 operand), found nothing.")),
                Some(format!("Try adding first operand"))
            ));
        }
        if let Some(_) = inst.src{
            if !right{
                return Some(RASMError::new(
                    Some(inst.lin),
                    ExType::Error,
                    Some(inst.to_string()),
                    Some(format!("Unexpected source (2 operand)!")),
                    Some(format!("Try removing second operand"))
                ));
            }
        }
    }
    if right{
        if let None = inst.src {
            return Some(RASMError::new(
                Some(inst.lin),
                ExType::Error,
                Some(inst.to_string()),
                Some(format!("Expected source (2 operand), found nothing.")),
                Some(format!("Try adding first operand"))
            ));
        }
        if let Some(_) = inst.dst{
            if !left{
                return Some(RASMError::new(
                    Some(inst.lin),
                    ExType::Error,
                    Some(inst.to_string()),
                    Some(format!("Unexpected destination (1 operand)!")),
                    Some(format!("Try removing first operand"))
                ));
            }
        }
    }
    return None;
}

fn chk_ins(inst: &ASTInstruction) -> Option<RASMError> {
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
