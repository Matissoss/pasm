// pasm - src/pre/par.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    error::Error,
    instruction::{Instruction, OperandOwned},
    mem::Mem,
    mnemonic::Mnemonic,
    num::Number,
    reg::Register,
    size::Size,
    stackvec::StackVec,
    symbol::SymbolRef,
};
use std::{str, str::FromStr};

#[derive(Debug, PartialEq)]
pub enum LineResult<'a> {
    Error(Error),
    Instruction(Instruction<'a>),
    Label(&'a str),
    Section(&'a str),
    Directive(&'a str, &'a str),
    None,
}

/// Parser
/// ---
/// EXPECTED INPUT:
///     - line is already stripped off comments and whitespace at start/end
#[cfg(not(feature = "refresh"))]
pub fn par<'a>(mut line: &'a str) -> LineResult<'a> {
    let line_bytes = line.as_bytes();
    if line_bytes.last() == Some(&b':') {
        return unsafe {
            LineResult::Label(str::from_utf8_unchecked(
                &line_bytes[0..line_bytes.len() - 1],
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
            } else if let Ok(amnem) = Mnemonic::from_str(line) {
                a_mnem = Some(amnem);
                line = "";
            }
            // then we go after operands and subexpressions
            let mut ins = Instruction::with_operands(StackVec::new());
            if let Some(a_mnem) = a_mnem {
                ins.set_addt(m);
                ins.mnemonic = a_mnem;
            } else {
                ins.mnemonic = m;
            }

            loop {
                let operand = if let Some((operand, rest)) = split_once_parser(line) {
                    line = rest.trim();
                    match par_operand(operand.trim()) {
                        Ok(o) => o,
                        Err(e) => {
                            // if we don't check that, parser thinks we're parsing memory operand
                            if line.split_whitespace().count() >= 2 {
                                return LineResult::Error(Error::new("operands (including subexpressions) need to be separated by ','", 5));
                            } else {
                                return LineResult::Error(e);
                            }
                        }
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
                            "bcst" => ins.set_evex_bcst(),
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
            LineResult::Directive(mnem, content)
        }
    } else if let Ok(mnem) = Mnemonic::from_str(line) {
        let mut instruction = Instruction::with_operands(StackVec::new());
        instruction.mnemonic = mnem;
        LineResult::Instruction(instruction)
    } else {
        LineResult::Directive(line, "")
    }
}

/// splits line more intelligently (so mov rax, ',' will work)          
fn split_once_parser(line: &str) -> Option<(&str, &str)> {
    let mut str_closure = false;
    for (i, b) in line.as_bytes().iter().enumerate() {
        if b == &b'"' || b == &b'\'' {
            str_closure = !str_closure;
        } else if b == &b',' && !str_closure {
            return Some((&line[0..i], &line[i + 1..]));
        } else if b == &b';' && !str_closure {
            return Some((&line[0..i], ""));
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
    if let Ok(n) = Number::from_str(slice) {
        Ok(ParserOperand::Imm(n))
    } else if let Ok(r) = Register::from_str(slice) {
        Ok(ParserOperand::Register(r))
    } else if slice.starts_with('{') && slice.ends_with('}') {
        Ok(ParserOperand::SubExpression(&slice[1..slice.len() - 1]))
    } else if slice.starts_with('"') && slice.ends_with('"') {
        Ok(ParserOperand::String(&slice[1..slice.len() - 1]))
    } else if let Ok(mut symbolref) = SymbolRef::from_str(slice) {
        symbolref.deref(false);
        Ok(ParserOperand::SymbolRef(symbolref))
    } else if let Some((sz, slice)) = slice.split_once(' ') {
        if sz.starts_with("q")
            || sz.starts_with("d")
            || sz.starts_with("w")
            || sz.starts_with("b")
            || sz.starts_with("x")
            || sz.starts_with("y")
            || sz.starts_with("z")
        {
            let sz = if let Ok(s) = Size::from_str(sz.trim()) {
                s
            } else {
                return Err(Error::new(
                    "failed to parse an operand: expected to find memory addressing here",
                    5,
                ));
            };

            if let Ok(mut m) = Mem::from_str(slice) {
                m.set_size(sz);
                Ok(ParserOperand::Mem(m))
            } else if let Ok(mut s) = SymbolRef::from_str(slice) {
                s.set_size(sz);
                s.deref(true);
                Ok(ParserOperand::SymbolRef(s))
            } else {
                Err(Error::new(
                    "failed to parse an operand: expected to find memory addressing",
                    5,
                ))
            }
        } else {
            Err(Error::new(
                format!("failed to parse operand \"{slice}\""),
                5,
            ))
        }
    } else {
        Err(Error::new(
            format!("failed to parse operand \"{slice}\""),
            5,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tsplit_once_parser_0() {
        let line = "',', .";
        assert_eq!(split_once_parser(line), Some(("','", " .")));
        let line = "string \"Hello, World!\"";
        assert_eq!(split_once_parser(line), None);
        // yeah, i know it's not real example, but it tests it?
        let line = "mov \",\", rax";
        assert_eq!(split_once_parser(line), Some(("mov \",\"", " rax")));
        let line = "rax, rcx";
        assert_eq!(split_once_parser(line), Some(("rax", " rcx")));
        let line = "this should be parsed; this SHOULD NOT BE!";
        assert_eq!(split_once_parser(line), Some(("this should be parsed", "")))
    }
    #[test]
    fn tparse_operands_1() {
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
    fn tparser_2() {
        let islice = "mov rax, rcx";
        let mut expected = Instruction::with_operands(StackVec::new());
        expected.push(OperandOwned::Register(Register::RAX));
        expected.push(OperandOwned::Register(Register::RCX));
        expected.mnemonic = Mnemonic::MOV;
        assert_eq!(par(islice), LineResult::Instruction(expected));
    }
}
