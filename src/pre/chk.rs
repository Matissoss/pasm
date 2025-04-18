// rasmx86_64 - chk.rs
// -------------------
// made by matissoss
// licensed under MPL
use crate::shr::{
    ins::Mnemonic as Mnm,
    ast::{
        AST,
        Instruction,
        Operand,
    },
    atype::*,
    error::RASMError,
    error::ExceptionType as ExType,
    num::Number,
    reg::Register
};

pub fn check_ast(file: &AST) -> Option<Vec<(String, Vec<RASMError>)>>{
    let mut errors : Vec<(String, Vec<RASMError>)> = Vec::new();

    let chk_ins : fn(&Instruction) -> Option<RASMError> = match file.bits{
        _ => check_ins64bit
    };

    for label in &file.labels{
        let mut errs = Vec::new();
        for inst in &label.inst{
            if let Some(mut err) = chk_ins(&inst){
                err.set_cont(inst.to_string());
                err.set_line(inst.line);
                errs.push(err);
            }
        }
        if !errs.is_empty(){
            errors.push((label.name.to_string(), errs));
        }
    }

    if errors.is_empty(){
        return None;
    }
    else {
        return Some(errors);
    }
}

fn check_ins64bit(ins: &Instruction) -> Option<RASMError>{
    return match ins.mnem{
        Mnm::PUSH => ot_chk(ins, (true, false), &[R16, R64, M16, M64, I8, I16, I32], &[], None),
        Mnm::POP  => ot_chk(ins, (true, false), &[R16, R64, M16, M64], &[], None),
        Mnm::MOV  => ot_chk(ins, (true, true),  &[R8, R16, R32, R64, M8, M16, M32, M64], 
                                                &[R8, R16, R32, R64, M8, M16, M32, M64, I8, I16, I32, I64], None),
        Mnm::SUB|Mnm::ADD|Mnm::CMP|Mnm::AND|Mnm::OR|Mnm::XOR  => ot_chk(ins, (true, true),  
            &[R8, R16, R32, R64, M8, M16, M32, M64], 
            &[R8, R16, R32, R64, M8, M16, M32, M64, I8, I16, I32], None),
        Mnm::IMUL          => {
            if let Some(_) = ins.src(){
                ot_chk(ins, (true, true), 
                    &[R8, R16, R32, R64, M8, M16, M32, M64], &[R16, R32, R64, M16, M32, M64], 
                    Some(&[I8, I16, I32]))
            }
            else {
                ot_chk(ins, (true, false),  &[R8, R16, R32, R64, M8, M16, M32, M64], &[], None)
            }
        },
        Mnm::SAL|Mnm::SHL|Mnm::SHR|Mnm::SAR => {
            if let Some(err) = operand_check(ins, (true, true)){
                return Some(err);
            }
            else {
                if let Some(err) = type_check(ins.dst().unwrap(), &[R8, R16, R32, R64, M8, M16, M32, M64], false){
                    return Some(err);
                }
                match ins.src().unwrap(){
                    Operand::Reg(Register::CL) => return None,
                    Operand::Imm(i) => {
                        return if let Some(u) = i.get_uint(){
                            match Number::squeeze_u64(u) {
                                Number::UInt8(_) => None,
                                _ => Some(RASMError::new(
                                    Some(ins.line),
                                    ExType::Error,
                                    Some(ins.to_string()),
                                    Some(format!("expected to found 8-bit number, found larger one instead")),
                                    Some(format!("sal/shl/shr/sar expect 8-bit number (like 1) or cl register"))
                                ))
                            }
                        }
                        else if let Some(i) = i.get_int(){
                            match Number::squeeze_i64(i) {
                                Number::Int8(_) => None,
                                _ => Some(RASMError::new(
                                    Some(ins.line),
                                    ExType::Error,
                                    Some(ins.to_string()),
                                    Some(format!("expected to found 8-bit number, found larger one instead")),
                                    Some(format!("sal/shl/shr/sar expect 8-bit number (like 1) or cl register"))
                                ))
                            }
                        }
                        else {
                            Some(RASMError::new(
                                Some(ins.line),
                                ExType::Error,
                                Some(ins.to_string()),
                                Some(format!("found non-compatible immediate for sal/shl/shr/sar instruction")),
                                Some(format!("sal/shl/shr/sar only allow for 8-bit number (like 255 or -128) or cl register"))
                            ))
                        }
                    },
                    _ => return Some(RASMError::new(
                        Some(ins.line),
                        ExType::Error,
                        Some(ins.to_string()),
                        Some(format!("source operand type mismatch, expected 8-bit number or cl register")),
                        Some(format!("consider changing source operand to 8-bit number or cl register"))
                    ))
                }
            }
        }
        Mnm::TEST => ot_chk(ins, (true, true), 
            &[R8, R16, R32, R64, M8, M16, M32, M64], &[I8, I16, I32, R8, R16, R32, R64], None),
        Mnm::DIV|Mnm::IDIV|Mnm::MUL|Mnm::DEC|Mnm::INC|Mnm::NEG|Mnm::NOT => {
            ot_chk(ins, (true, false), &[R8, R16, R32, R64, M8, M16, M32, M64], &[], None)
        },
        Mnm::JMP|Mnm::JE|Mnm::JNE|Mnm::JZ|Mnm::JNZ|Mnm::CALL|Mnm::JL|Mnm::JLE|Mnm::JG|Mnm::JGE => {
            ot_chk(ins, (true, false), &[AType::Sym], &[], None)
        },
        Mnm::LEA => ot_chk(ins, (true, true), &[R16, R32, R64], &[AType::Sym], None),
        Mnm::SYSCALL|Mnm::RET => ot_chk(ins, (false, false), &[], &[], None),
        /*
        _ => Some(RASMError::new(
            Some(ins.line),
            ExType::Error,
            Some(ins.to_string()),
            Some(format!("Tried to use unsupported instruction")),
            None
        )),
        */
    }
}

fn ot_chk(ins: &Instruction, ops: (bool, bool), dstt: &[AType], srct: &[AType], third_op: Option<&[AType]>) -> Option<RASMError>{
    if let Some(e) = operand_check(ins, ops){
        return Some(e);
    }
    else {
        if ops.0 {
            if let Some(err) = type_check(ins.dst().unwrap(), dstt, false){
                return Some(err);
            }
        }
        if ops.1{
            if let Some(err) = type_check(ins.src().unwrap(), srct, true){
                return Some(err);
            }
        }
        if let Some(thrd_types) = third_op{
            if let Some(thrd_op) = ins.oprs.get(2){
                return type_check(thrd_op, thrd_types, false);
            }
        }
        if ops == (true, true){
            if let Some(err) = prev_mm(ins){
                return Some(err);
            }
            else {
                return size_chk(ins);
            }
        }
        return None;
    }
}

fn operand_check(ins: &Instruction, ops: (bool, bool)) -> Option<RASMError>{
    match (ins.dst(), ops.0){
        (None, false)|(Some(_), true) => {},
        (Some(_), false) => return Some(RASMError::new(
            None,
            ExType::Error,
            None,
            Some(format!("Unexpected destination operand found: expected none, found some")),
            Some(format!("Consider removing destination operand"))
        )),
        (None, true) => return Some(RASMError::new(
            None,
            ExType::Error,
            None,
            Some(format!("Expected destination operand, found nothing")),
            Some(format!("Consider adding destination operand"))
        ))
    };
    match (ins.src(), ops.1) {
        (None, false)|(Some(_), true) => return None,
        (Some(_), false) => return Some(RASMError::new(
            None,
            ExType::Error,
            None,
            Some(format!("Unexpected source operand found: expected none, found some")),
            Some(format!("Consider removing source operand"))
        )),
        (None, true) => return Some(RASMError::new(
            None,
            ExType::Error,
            None,
            Some(format!("Expected source operand, found nothing")),
            Some(format!("Consider adding source operand"))
        ))
    }
}

fn type_check(operand: &Operand, accepted: &[AType], src_op: bool) -> Option<RASMError>{
    if accepted.contains(&operand.atype()){
        return None;
    }
    else {
        let err = RASMError::new(
            None,
            ExType::Error,
            None,
            Some(format!("{} operand doesn't match any of expected types: {:?}",
                    if src_op {"Source"} else {"Destination"}, accepted
            )),
            Some(format!("Consider changing {} operand to expected type or removing instruction",
                    if src_op {"source"} else {"destination"}
            ))
        );
        
        if let Operand::Imm(imm) = operand{
            match imm{
                Number::UInt64(n) => {
                    if accepted.contains(&Number::squeeze_u64(*n).atype()){
                        return None
                    }
                },
                Number::Int64(n) => {
                    if accepted.contains(&Number::squeeze_i64(*n).atype()){
                        return None
                    }
                }
                _ => {}
            }
        }
        return Some(err);
    }
}
fn size_chk(ins: &Instruction) -> Option<RASMError>{
    let dst = ins.dst().unwrap();
    let src = ins.src().unwrap();
    // should work (i hope so)
    return match (dst.atype(), src.atype()){
        (AType::Reg(s0)|AType::Mem(s0), AType::Imm(s1)) => {
            if s1 <= s0{
                None
            }
            else {
                return if !ins.mnem.allows_diff_size(Some(s0), Some(s1)){
                    Some(RASMError::new(
                        Some(ins.line),
                        ExType::Error,
                        Some(ins.to_string()),
                        Some(format!("Tried to use immediate that is too large for destination operand")),
                        Some(format!("Consider changing immediate to fit inside {} bytes", s0 as u8))
                    ))
                }
                else {
                    None
                }
            }
        }
        (AType::Reg(s0)|AType::Mem(s0), AType::Reg(s1)|AType::Mem(s1)) => {
            if s1 == s0{
                None
            }
            else {
                return if !ins.mnem.allows_diff_size(Some(s0), Some(s1)){
                    Some(RASMError::new(
                        Some(ins.line),
                        ExType::Error,
                        Some(ins.to_string()),
                        Some(format!("Tried to use operand that cannot be used for destination operand")),
                        Some(format!("Consider changing operand to be {} byte", s0 as u8))
                    ))
                }
                else {
                    None
                }
            }
        },
        _ => None
    }
}

// prevent memory-memory
fn prev_mm(ins: &Instruction) -> Option<RASMError>{
    if ins.mnem.allows_mem_mem(){
        return None;
    }

    if let (Some(Operand::Mem(_)), Some(Operand::Mem(_))) = (ins.dst(), ins.src()){
        return Some(RASMError::new(
            Some(ins.line),
            ExType::Error,
            Some(ins.to_string()),
            Some(format!("Tried to perform illegal operation on instruction that doesn't support memory-memory operand combo")),
            Some(format!("Consider using register to store memory and then using with other memory instead"))
        ))
    }
    return None;
}
