// pasm - src/pre/par.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    ast::{Instruction, OperandOwned},
    error::Error,
    ins::Mnemonic,
    mem::Mem,
    num::Number,
    reg::Register,
    size::Size,
    smallvec::SmallVec,
    symbol::SymbolRef,
};
use std::{collections::HashMap, str, str::FromStr};

#[derive(Debug, PartialEq, Default)]
pub struct ParserStatus<'a> {
    pub attributes: HashMap<&'a str, &'a str>,
}

#[derive(Debug, PartialEq)]
pub enum LineResult<'a> {
    Error(Error),
    Instruction(Instruction<'a>),
    Label(&'a str),
    Section(&'a str),
    Directive,
    None,
}

/// Parser
/// ---
/// EXPECTED INPUT:
///     - line is already stripped off comments and whitespace at start/end
pub fn par<'a>(parser_status: *mut ParserStatus<'a>, mut line: &'a str) -> LineResult<'a> {
    let parser_status = unsafe { &mut *parser_status };
    let line_bytes = line.as_bytes();
    if line_bytes.last() == Some(&b':') {
        return unsafe {
            LineResult::Label(str::from_utf8_unchecked(
                &line_bytes[0..line_bytes.len() - 2],
            ))
        };
    }
    if let Some((mnem, content)) = line.split_once(' ') {
        if let Ok(m) = Mnemonic::from_str(mnem) {
            let mut a_mnem: Option<Mnemonic> = None;
            // first we need to detect if we have additional mnemonic
            line = content;
            if let Some((a_mnem_str, rest)) = line.split_once(' ') {
                if let Ok(mnem) = Mnemonic::from_str(a_mnem_str) {
                    a_mnem = Some(mnem);
                    line = rest;
                }
            }
            // then we go after operands and subexpressions
            let mut ins = Instruction::with_operands(SmallVec::new());
            ins.mnemonic = m;
            if let Some(a_mnem) = a_mnem {
                ins.set_addt(a_mnem);
            }
            loop {
                let operand = if let Some((operand, rest)) = split_once_intelligent(line) {
                    line = rest.trim();
                    match par_operand(operand.trim()) {
                        Ok(o) => o,
                        Err(e) => return LineResult::Error(e),
                    }
                } else {
                    if line.is_empty() {
                        break;
                    }
                    match par_operand(line.trim()) {
                        Ok(o) => {
                            line = "";
                            o
                        }
                        Err(e) => return LineResult::Error(e),
                    }
                };
                match operand {
                    ParserOperand::String(s) => ins.push(OperandOwned::String(
                        std::mem::ManuallyDrop::new(Box::new(s)),
                    )),
                    ParserOperand::SymbolRef(s) => ins.push(OperandOwned::Symbol(
                        std::mem::ManuallyDrop::new(Box::new(s)),
                    )),
                    ParserOperand::Mem(m) => ins.push(OperandOwned::Mem(m)),
                    ParserOperand::Register(r) => ins.push(OperandOwned::Register(r)),
                    ParserOperand::Imm(i) => ins.push(OperandOwned::Imm(i)),
                    ParserOperand::SubExpression(s) => {
                        match s {
                            // APX
                            "apx" => ins.apx_set_default(),
                            "apx-evex" => ins.apx_evex_set_apx_extension(true),
                            "of" => ins.apx_eevex_cond_set_of(),
                            "cf" => ins.apx_eevex_cond_set_cf(),
                            "zf" => ins.apx_eevex_cond_set_zf(),
                            "sf" => ins.apx_eevex_cond_set_sf(),
                            "rex2" => ins.apx_set_rex2(),
                            "eevex" => ins.apx_set_eevex(),
                            "nf" => ins.apx_set_leg_nf(),
                            "vex-nf" => ins.apx_set_vex_nf(),
                            // AVX-512
                            "k0" => ins.set_evex_mask(0b000),
                            "k1" => ins.set_evex_mask(0b001),
                            "k2" => ins.set_evex_mask(0b010),
                            "k3" => ins.set_evex_mask(0b011),
                            "k4" => ins.set_evex_mask(0b100),
                            "k5" => ins.set_evex_mask(0b101),
                            "k6" => ins.set_evex_mask(0b110),
                            "k7" => ins.set_evex_mask(0b111),
                            "sae" => ins.set_evex_sae(),
                            "er" => ins.set_evex_er(0b001),
                            "rn-sae" => ins.set_evex_er(0b001),
                            "rd-sae" => ins.set_evex_er(0b010),
                            "ru-sae" => ins.set_evex_er(0b011),
                            "rz-sae" => ins.set_evex_er(0b100),
                            "z" => ins.set_evex_z(),
                            "evex" => ins.set_evex(),
                            "vex" => ins.set_vex(),
                            _ => {
                                return LineResult::Error(Error::new(
                                    format!(
                                    "you tried to use unknown/unsupported subexpression: \"{s}\""
                                ),
                                    4,
                                ))
                            }
                        }
                    }
                }
            }

            LineResult::Instruction(ins)
        } else if mnem == "section" {
            LineResult::Section(content)
        } else {
            parser_status.attributes.insert(mnem, content);
            LineResult::Directive
        }
    } else if let Ok(mnem) = Mnemonic::from_str(line) {
        let mut instruction = Instruction::with_operands(SmallVec::new());
        instruction.mnemonic = mnem;
        LineResult::Instruction(instruction)
    } else {
        LineResult::Error(Error::new("couldn't parse this line for some reason", 3))
    }
}

/// splits line more intelligently (so mov rax, ',' will work)          
fn split_once_intelligent(line: &str) -> Option<(&str, &str)> {
    let mut str_closure = false;
    for (i, b) in line.as_bytes().iter().enumerate() {
        if b == &b'"' || b == &b'\'' {
            str_closure = !str_closure;
        } else if b == &b',' && !str_closure {
            return Some((&line[0..i], &line[i + 1..]));
        }
    }
    None
}

#[derive(Debug, PartialEq)]
enum ParserOperand<'a> {
    SubExpression(&'a str),
    String(&'a str),
    Imm(Number),
    Register(Register),
    Mem(Mem),
    SymbolRef(SymbolRef<'a>),
}

fn par_operand<'a>(slice: &'a str) -> Result<ParserOperand<'a>, Error> {
    if let Some(n) = Number::from_str(slice) {
        Ok(ParserOperand::Imm(n))
    } else if let Ok(r) = Register::from_str(slice) {
        Ok(ParserOperand::Register(r))
    } else if slice.starts_with('{') && slice.ends_with('}') {
        Ok(ParserOperand::SubExpression(&slice[1..slice.len() - 1]))
    } else if slice.starts_with('"') && slice.ends_with('"') {
        Ok(ParserOperand::String(&slice[1..slice.len() - 1]))
    } else if let Some((sz, slice)) = slice.split_once(' ') {
        let sz = if let Ok(s) = Size::from_str(sz.trim()) {
            s
        } else {
            return Err(Error::new(
                "expected to find size directive here, found something else",
                5,
            ));
        };

        if let Ok(m) = Mem::new(slice, sz) {
            Ok(ParserOperand::Mem(m))
        } else {
            Err(Error::new(
                "expected a memory address, found something else",
                5,
            ))
        }
    } else if Mem::new(slice, Size::Any).is_ok() {
        Err(Error::new(
                "you tried to use memory addressing without size directive, which is forbidden in PASM at the moment",
                5
        ))
    } else {
        Ok(ParserOperand::SymbolRef(
            SymbolRef::from_str(slice).unwrap(),
        ))
    }
}

#[cfg(test)]
mod partest {
    use super::*;
    #[test]
    fn si_test() {
        let line = "',', .";
        assert_eq!(split_once_intelligent(line), Some(("','", " .")));
        let line = "string \"Hello, World!\"";
        assert_eq!(split_once_intelligent(line), None);
        // yeah, i know it's not real example, but it tests it?
        let line = "mov \",\", rax";
        assert_eq!(split_once_intelligent(line), Some(("mov \",\"", " rax")));
        let line = "rax, rcx";
        assert_eq!(split_once_intelligent(line), Some(("rax", " rcx")));
    }
    #[test]
    fn po_test() {
        let slice = "rax";
        assert_eq!(
            par_operand(slice),
            Ok(ParserOperand::Register(Register::RAX))
        );
        let slice = "qword [rax + 10]";
        if let Ok(ParserOperand::Mem(_)) = par_operand(slice) {
        } else {
            panic!("didn't parse into mem");
        }
        let slice = "\"string\"";
        assert_eq!(par_operand(slice), Ok(ParserOperand::String("string")));
        let slice = "{subexpr}";
        assert_eq!(
            par_operand(slice),
            Ok(ParserOperand::SubExpression("subexpr"))
        );
    }
    #[test]
    fn partest() {
        let islice = "mov rax, rcx";
        let mut expected = Instruction::with_operands(SmallVec::new());
        expected.push(OperandOwned::Register(Register::RAX));
        expected.push(OperandOwned::Register(Register::RCX));
        let mut parser_status = ParserStatus {
            attributes: HashMap::new(),
        };
        expected.mnemonic = Mnemonic::MOV;
        assert_eq!(
            par(&mut parser_status, islice),
            LineResult::Instruction(expected)
        );
    }
}
