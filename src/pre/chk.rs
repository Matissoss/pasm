// rasmx86_64 - src/pre/chk.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::core::rex::gen_rex;
use crate::shr::{
    ins::Mnemonic as Mnm,
    ast::{
        AST,
        Instruction,
        Operand,
    },
    atype::*,
    error::RASMError,
    num::Number,
    reg::Register,
    size::Size
};

pub fn check_ast(file: &AST) -> Option<Vec<(String, Vec<RASMError>)>>{
    let mut errors : Vec<(String, Vec<RASMError>)> = Vec::new();

    let chk_ins : fn(&Instruction) -> Option<RASMError> = match file.bits{
        Some(16)|Some(32) => check_ins32bit,
        Some(64)|_        => check_ins64bit
    };

    for label in &file.labels{
        let mut errs = Vec::new();
        for inst in &label.inst{
            if let Some(mut err) = chk_ins(&inst){
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

fn check_ins32bit(ins: &Instruction) -> Option<RASMError>{
    if gen_rex(ins, false).is_some(){
        return Some(RASMError::new(
            Some(ins.line),
            Some("Instruction needs rex prefix, which is forbidden in protected/compat mode (bits 32)".to_string()),
            None
        ));
    }
    return match ins.mnem{
        Mnm::PUSH => ot_chk(ins, &[
            (&[R16,R32,M16,M32,I8,I16,I32,SR], Optional::Needed)
        ], &[], &[]),
        Mnm::POP => ot_chk(ins, &[(&[R16, R32, M16, M32, SR], Optional::Needed)], &[], &[]),
        Mnm::MOV  => ot_chk(ins, &[
            (&[R8,R16,R32,M8,M16,M32,SR,CR], Optional::Needed),
            (&[R8,R16,R32,M8,M16,M32,I8,I16,I32,SR,CR], Optional::Needed)],
            &[(MA, MA), (R32, SR), (M32, SR), (M8, SR), 
              (R8, SR), (SR, R32), (SR, R8), (SR, IA),
              (SR, M32), (SR, M8), (CR, IA), (CR, R8), (CR, R16),
              (R16, CR), (R8, CR), (CR, MA), (MA, CR), (DR, IA), (DR, R8), (DR, R16),
              (DR, R32), (R16, DR), (R8, DR), (DR, MA), 
              (MA, DR), (R8, DR), (DR, MA), (MA, DR)
            ],
            &[]
        ),
        Mnm::SUB|Mnm::ADD|Mnm::CMP|Mnm::AND|Mnm::OR|Mnm::XOR => ot_chk(ins, 
            &[(&[R8, R16, R32, M8, M16, M32], Optional::Needed),
              (&[R8, R16, R32, M8, M16, M32, I8, I16, I32], Optional::Needed)
            ], &[(MA, MA)], &[]),
        Mnm::IMUL => ot_chk(ins, &[
            (&[R8, R16, R32, M8, M16, M32], Optional::Needed), (&[R16, R32, M16, M32], Optional::Optional),
            (&[I8, I16, I32], Optional::Optional)
        ], &[(MA, MA)], &[]),
        Mnm::SAL|Mnm::SHL|Mnm::SHR|Mnm::SAR => {
            if let Some(err) = operand_check(ins, (true, true)){
                return Some(err);
            }
            else {
                if let Some(err) = type_check(ins.dst().unwrap(), &[R8, R16, R32, M8, M16, M32], 1){
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
                                    Some(format!("expected to found 8-bit number, found larger one instead")),
                                    Some(format!("sal/shl/shr/sar expect 8-bit number (like 1) or cl register"))
                                ))
                            }
                        }
                        else {
                            Some(RASMError::new(
                                Some(ins.line),
                                Some(format!("found non-compatible immediate for sal/shl/shr/sar instruction")),
                                Some(format!("sal/shl/shr/sar only allow for 8-bit number (like 255 or -128) or cl register"))
                            ))
                        }
                    },
                    _ => return Some(RASMError::new(
                        Some(ins.line),
                        Some(format!("source operand type mismatch, expected 8-bit number or cl register")),
                        Some(format!("consider changing source operand to 8-bit number or cl register"))
                    ))
                }
            }
        },
        Mnm::TEST => ot_chk(ins, 
            &[(&[R8, R16, R32, M8, M16, M32], Optional::Needed),
              (&[I8, I16, I32, R8, R16, R32], Optional::Needed)],
            &[], &[]
        ),
        Mnm::DIV|Mnm::IDIV|Mnm::MUL|Mnm::DEC|Mnm::INC|Mnm::NEG|Mnm::NOT => {
            ot_chk(ins, &[
                (&[R8, R16, R32, M8, M16, M32], Optional::Needed)
            ], &[], &[])
        },
        Mnm::JMP|Mnm::CALL => ot_chk(ins, 
            &[(&[AType::Sym, R32, R16, M32, M16], Optional::Needed)], &[], &[]),
        Mnm::JE|Mnm::JNE|Mnm::JZ|Mnm::JNZ|Mnm::JL|Mnm::JLE|Mnm::JG|Mnm::JGE => {
            ot_chk(ins, &[(&[AType::Sym], Optional::Needed)], &[], &[])
        },
        Mnm::LEA => ot_chk(ins, &[(&[R16, R32], Optional::Needed), (&[AType::Sym], Optional::Needed)], &[], &[]),
        Mnm::SYSCALL|Mnm::RET|Mnm::NOP => ot_chk(ins, &[], &[], &[]),
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

fn check_ins64bit(ins: &Instruction) -> Option<RASMError>{
    return match ins.mnem{
        Mnm::PUSH => ot_chk(ins, &[
            (&[R16,R64,M16,M64,I8,I16,I32, SR], Optional::Needed)
        ], &[], &[]),
        Mnm::POP => ot_chk(ins, &[(&[R16, R64, M16, M64, SR], Optional::Needed)], &[], &[]),
        Mnm::MOV  => ot_chk(ins, &[
            (&[R8,R16,R32,R64,M8,M16,M32,M64,SR,CR,DR], Optional::Needed),
            (&[R8,R16,R32,R64,M8,M16,M32,M64,I8,I16,I32,I64,SR,CR, DR], Optional::Needed)],
            &[(MA, MA), (R32, SR), (M32, SR), (M8, SR), 
              (R8, SR), (SR, R32), (SR, R8), (SR, IA),
              (SR, M32), (SR, M8), (CR, IA), (CR, R8), (CR, R16),
              (CR, R32), (R16, CR), (DR, IA), (DR, R8), (DR, R16),
              (DR, R32), (R16, DR), (R8, DR), (DR, MA), 
              (MA, DR), (R8, DR), (DR, MA), (MA, DR)
            ],
            &[]
        ),
        Mnm::SUB|Mnm::ADD|Mnm::CMP|Mnm::AND|Mnm::OR|Mnm::XOR => ot_chk(ins, 
            &[(&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed),
              (&[R8, R16, R32, R64, M8, M16, M32, M64, I8, I16, I32], Optional::Needed)
            ], &[(MA, MA)], &[]),
        Mnm::IMUL => ot_chk(ins, &[
            (&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed), (&[R16, R32, R64, M16, M32, M64], Optional::Optional),
            (&[I8, I16, I32], Optional::Optional)
        ], &[(MA, MA)], &[]),
        Mnm::SAL|Mnm::SHL|Mnm::SHR|Mnm::SAR => {
            if let Some(err) = operand_check(ins, (true, true)){
                return Some(err);
            }
            else {
                if let Some(err) = type_check(ins.dst().unwrap(), &[R8, R16, R32, R64, M8, M16, M32, M64], 1){
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
                                    Some(format!("expected to found 8-bit number, found larger one instead")),
                                    Some(format!("sal/shl/shr/sar expect 8-bit number (like 1) or cl register"))
                                ))
                            }
                        }
                        else {
                            Some(RASMError::new(
                                Some(ins.line),
                                Some(format!("found non-compatible immediate for sal/shl/shr/sar instruction")),
                                Some(format!("sal/shl/shr/sar only allow for 8-bit number (like 255 or -128) or cl register"))
                            ))
                        }
                    },
                    _ => return Some(RASMError::new(
                        Some(ins.line),
                        Some(format!("source operand type mismatch, expected 8-bit number or cl register")),
                        Some(format!("consider changing source operand to 8-bit number or cl register"))
                    ))
                }
            }
        },
        Mnm::TEST => ot_chk(ins, 
            &[(&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed),
              (&[I8, I16, I32, R8, R16, R32, R64], Optional::Needed)],
            &[], &[]
        ),
        Mnm::DIV|Mnm::IDIV|Mnm::MUL|Mnm::DEC|Mnm::INC|Mnm::NEG|Mnm::NOT => {
            ot_chk(ins, &[
                (&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed)
            ], &[], &[])
        },
        Mnm::JMP|Mnm::CALL => ot_chk(ins, 
            &[(&[AType::Sym, R64, M64], Optional::Needed)], &[], &[]),
        Mnm::JE|Mnm::JNE|Mnm::JZ|Mnm::JNZ|Mnm::JL|Mnm::JLE|Mnm::JG|Mnm::JGE => {
            ot_chk(ins, &[(&[AType::Sym], Optional::Needed)], &[], &[])
        },
        Mnm::LEA => ot_chk(ins, &[(&[R16, R32, R64], Optional::Needed), (&[AType::Sym], Optional::Needed)], &[], &[]),
        Mnm::SYSCALL|Mnm::RET|Mnm::NOP => ot_chk(ins, &[], &[], &[]),
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

#[derive(PartialEq)]
enum Optional{
    Needed,
    Optional,
}

fn ot_chk(ins: &Instruction, ops: &[(&[AType], Optional)], forb: &[(AType, AType)], addt: &[Mnm]) -> Option<RASMError>{
    if let Some(err) = addt_chk(ins, addt){
        return Some(err);
    }
    let mut idx = 0;
    if ops.len() == 0  && ins.oprs.len() != 0{
        return Some(RASMError::new(
            Some(ins.line),
            Some("Instruction doesn't accept any operand, but you tried to use one anyways".to_string()),
            None
        ));
    }
    for allowed in ops{
        if let Some(op) = ins.oprs.get(idx){
            if let Some(err) = type_check(op, allowed.0, idx){
                return Some(err);
            }
        }
        else {
            if allowed.1 == Optional::Needed{
                return Some(RASMError::new(
                    Some(ins.line),
                    Some(format!("Needed operand not found at index {}", idx)),
                    None
                ));
            }
        }
        idx += 1;
    }
    if ops.len() == 2{
        if let Some(err) = size_chk(ins){
            return Some(err);
        }
    }
    if let Some(err) = forb_chk(ins, forb){
        return Some(err);
    }
    None
}

fn forb_chk(ins: &Instruction, forb: &[(AType, AType)]) -> Option<RASMError>{
    let dst_t = if let Some(dst) = ins.dst(){
        dst.atype()
    } else {return None};
    let src_t = if let Some(src) = ins.src(){
        src.atype()
    } else {return None};
    for f in forb{
        if (dst_t, src_t) == *f{
            return Some(RASMError::new(
                Some(ins.line),
                Some(format!("Destination and Source operand have forbidden combination: ({:?}, {:?})", f.0, f.1)),
                None
            ));
        }
    }
    None
}

fn operand_check(ins: &Instruction, ops: (bool, bool)) -> Option<RASMError>{
    match (ins.dst(), ops.0){
        (None, false)|(Some(_), true) => {},
        (Some(_), false) => return Some(RASMError::new(
            None,
            Some(format!("Unexpected destination operand found: expected none, found some")),
            Some(format!("Consider removing destination operand"))
        )),
        (None, true) => return Some(RASMError::new(
            None,
            Some(format!("Expected destination operand, found nothing")),
            Some(format!("Consider adding destination operand"))
        ))
    };
    match (ins.src(), ops.1) {
        (None, false)|(Some(_), true) => return None,
        (Some(_), false) => return Some(RASMError::new(
            None,
            Some(format!("Unexpected source operand found: expected none, found some")),
            Some(format!("Consider removing source operand"))
        )),
        (None, true) => return Some(RASMError::new(
            None,
            Some(format!("Expected source operand, found nothing")),
            Some(format!("Consider adding source operand"))
        ))
    }
}

fn type_check(operand: &Operand, accepted: &[AType], idx: usize) -> Option<RASMError>{
    if accepted.contains(&operand.atype()){
        return None;
    }
    else {
        let err = RASMError::new(
            None,
            Some(format!("{} operand doesn't match any of expected types: {:?}",
                 match idx {
                     1 => "Destination".to_string(),
                     2 => "Source".to_string(),
                     _ => idx.to_string()
                 }, accepted
            )),
            Some(format!("Consider changing {} operand to expected type or removing instruction",
                 match idx {
                     1 => "Destination".to_string(),
                     2 => "Source".to_string(),
                     _ => idx.to_string()
                 }
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

    if let Operand::CtrReg(_) = dst{
        return None;
    }
    if let Operand::CtrReg(_) = src{
        return None;
    }
    // should work (i hope so)
    return match (dst.atype(), src.atype()){
        (AType::Reg(s0)|AType::Mem(s0)|AType::Segment(s0), AType::Imm(s1)) => {
            if s1 <= s0{
                None
            }
            else {
                return if !ins.mnem.allows_diff_size(Some(s0), Some(s1)){
                    Some(RASMError::new(
                        Some(ins.line),
                        Some(format!("Tried to use immediate that is too large for destination operand")),
                        Some(format!("Consider changing immediate to fit inside {} bits",<Size as Into<u8>>::into(s0) as u16 * 8))
                    ))
                }
                else {
                    None
                }
            }
        },
        (AType::Reg(s0)|AType::Mem(s0)|AType::Segment(s0), AType::Reg(s1)|AType::Mem(s1)|AType::Segment(s1)) => {
            if s1 == s0{
                None
            }
            else {
                return if !ins.mnem.allows_diff_size(Some(s0), Some(s1)){
                    Some(RASMError::new(
                        Some(ins.line),
                        Some(format!("Tried to use operand that cannot be used for destination operand")),
                        Some(format!("Consider changing operand to be {}-bit", <Size as Into<u8>>::into(s0) as u16 * 8))
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

fn addt_chk(ins: &Instruction, accpt_addt: &[Mnm]) -> Option<RASMError>{
    if let Some(addt) = &ins.addt{
        for a in addt {
            if !find_bool(accpt_addt, &a){
                return Some(RASMError::new(
                    Some(ins.line),
                    Some(format!("Use of forbidden additional mnemonic: {}", a.to_string())),
                    None
                ));
            }
        }
    }
    None
}

fn find_bool(addts: &[Mnm], searched: &Mnm) -> bool{
    for addt in addts{
        if searched == addt{
            return true;
        }
    }
    false
}
