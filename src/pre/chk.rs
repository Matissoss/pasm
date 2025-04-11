// rasmx86_64 - chk.rs
// -------------------
// made by matissoss
// licensed under MPL
use crate::conf::PREFIX_REG;
use crate::shr::{
    ins::Mnemonic as Ins,
    ast::{
        AST,
        Instruction,
        AsmType,
        ToAsmType,
        Operand,
        Label,
        AsmTypes
    },
    error::RASMError,
    error::ExceptionType as ExType
};

pub fn check_file(file: &AST) -> Option<Vec<(String, Vec<RASMError>)>>{
    let mut errors : Vec<(String, Vec<RASMError>)> = Vec::new();

    for label in &file.labels{
        if let Some(errs) = check_label(label){
            errors.push((label.name.clone(), errs));
        }
    }
    if errors.is_empty() {
        return None;
    }
    else{
        return Some(errors);
    }
}

fn size_check(inst: &Instruction) -> Option<RASMError>{
    if let (Some(d), Some(s)) = (&inst.dst(), &inst.src()){
        if let Operand::Imm(s_i) = s{
            if d.size() < s_i.size(){
                return Some(RASMError::new(
                    Some(inst.line),
                    ExType::Error,
                    Some(inst.to_string()),
                    Some(format!("Destination has smaller size than immediate you tried to assign for destination!")),
                    Some(format!("Make source (immediate) fit in range of {} number", d.size()))
                ));
            }
            return None;
        }
        if let Operand::Mem(m_i) = s{
            if d.size() > m_i.size(){
                return Some(RASMError::new(
                    Some(inst.line),
                    ExType::Error,
                    Some(inst.to_string()),
                    Some(format!("Illegal operation: tried to assign {} value from address into {} destination", 
                        m_i.size() as u16 * 8, d.size() as u16 * 8)),
                    Some(format!("Consider using smaller register, like `{}cl` instead of `{}rcx` for 8-bit value.", PREFIX_REG, PREFIX_REG))
                ))
            }
        }
        if d.size() != s.size() && !inst.mnem.allows_diff_size(Some(d.size()), Some(s.size())){
            return Some(RASMError::new(
                Some(inst.line),
                ExType::Error,
                Some(inst.to_string()),
                Some(format!("Found 2 different size for operands and instruction doesn't allow for it: {}B for dst, {}B for src",
                    d.size(), s.size())),
                Some(format!("Try using same size operand"))
            ));
        }
    }
    return None;
}

// new checks (more readable)
pub fn check_label(lbl: &Label) -> Option<Vec<RASMError>> {
    let mut errors = Vec::new();

    for ins in &lbl.inst{
        if let Some(err) = entry_check(ins){
            errors.push(err);
        }
        else if let Some(err) = type_check(ins){
            errors.push(err);
        }
        else if let Some(err) = size_check(ins){
            errors.push(err);
        }
    }

    if errors.is_empty() {
        None
    }
    else {
        Some(errors)
    }
}

// checks for operands appearance
fn entry_check(ins: &Instruction) -> Option<RASMError>{
    match &ins.mnem{
         Ins::CALL|Ins::JE |Ins::JNE
        |Ins::JL  |Ins::JLE|Ins::JG 
        |Ins::JGE |Ins::JMP|Ins::JZ
        |Ins::PUSH|Ins::POP|Ins::JNZ    => has_operands(&ins, true , false),
        Ins::SYSCALL|Ins::RET           => has_operands(&ins, false, false),
        _                               => has_operands(&ins, true , true ),
    }
}

fn has_operands(ins: &Instruction, dst_bool: bool, src_bool: bool) -> Option<RASMError>{
    match (&ins.dst(), dst_bool) {
        (Some(dst), false) => {
            return Some(RASMError::new(
                Some(ins.line),
                ExType::Error,
                Some(ins.to_string()),
                Some(format!("Unexpected destination (1 operand): {}", dst.to_string())),
                Some(format!("Try removing first operand"))
            ));
        },
        (None, true) => {
            return Some(RASMError::new(
                Some(ins.line),
                ExType::Error,
                Some(ins.to_string()),
                Some(format!("Expected destination (first operand), found nothing.")),
                Some(format!("Try adding first operand"))
            ));
        },
        _ => {}
    }
    match (&ins.src(), src_bool) {
        (Some(src), false) => {
            return Some(RASMError::new(
                Some(ins.line),
                ExType::Error,
                Some(ins.to_string()),
                Some(format!("Unexpected source (second operand): {}", src.to_string())),
                Some(format!("Try removing second operand"))
            ));
        },
        (None, true) => {
            return Some(RASMError::new(
                Some(ins.line),
                ExType::Error,
                Some(ins.to_string()),
                Some(format!("Expected source (second operand), found nothing.")),
                Some(format!("Try adding second operand"))
            ));
        },
        _ => {}
    }
    return None;
}

fn type_check(ins: &Instruction) -> Option<RASMError>{
    let err_dst = "ins.dst == Some; assertion failed";
    let err_src = "ins.src == Some; assertion failed";
    
    match ins.mnem{
        Ins::JMP|Ins::CALL|Ins::JE  |Ins::JNE|
        Ins::JL |Ins::JLE |Ins::JG  |Ins::JGE|
        Ins::JZ |Ins::JNZ => {
            if let Some(mut err) = opch(ins.dst().expect(err_dst).clone(), &[AsmType::LabelRef]){
                err.set_line(ins.line);
                err.set_cont(ins.to_string());
                return Some(err);
            } 
            else {
                return None;
            }
        },
        Ins::PUSH => {
            if let Some(mut err) = opch(ins.dst().expect(err_dst).clone(), &[AsmType::Mem,AsmType::Reg,AsmType::Imm]){
                err.set_line(ins.line);
                err.set_cont(ins.to_string());
                return Some(err);
            } 
            else {
                return None;
            }
        }
        Ins::RET|Ins::SYSCALL => None,
        _ => {
            if let (AsmType::Mem, AsmType::Mem, false) = (ins.dst().clone().expect(err_dst).asm_type(), 
                                                          ins.src().clone().expect(err_src).asm_type(), 
                                                          ins.mnem.allows_mem_mem())
            {
                return Some(RASMError::new(
                    Some(ins.line),
                    ExType::Error,
                    Some(ins.to_string()),
                    Some(format!("Tried to use memory-memory operand combination and instruction doesn't support that")),
                    Some(format!("Consider moving value into register and then comparing with memory"))
                ));
            }

            let dst_types = [AsmType::Mem, AsmType::Reg];
            let src_types = [AsmType::Reg, AsmType::Mem, AsmType::Imm, AsmType::ConstRef];
            if let Some(mut err) = opch(ins.dst().expect(err_dst).clone(), &dst_types){
                err.set_line(ins.line);
                err.set_cont(ins.to_string());
                return Some(err);
            }
            else if let Some(mut err) = opch(ins.src().expect(err_src).clone(), &src_types){
                err.set_line(ins.line);
                err.set_cont(ins.to_string());
                return Some(err);
            }
            else {
                return None;
            }
        }
    }
}

// operand check. we can use inline as it is just short if statement
#[inline(always)]
fn opch(op: Operand, types: &[AsmType]) -> Option<RASMError> {
    if !types.contains(&op.asm_type()){
        return Some(RASMError::new(
            None,
            ExType::Error,
            None,
            Some(
                format!("Unexpected type found, expected either one of types: {}, found: {:?}", 
                    AsmTypes(types.to_vec()).to_string(),
                    op.asm_type().to_string(),
                )
            ),
            Some(
                format!("Consider changing operand to one of supported types")
            )
        ));
    }
    else {
        return None;
    }
}
