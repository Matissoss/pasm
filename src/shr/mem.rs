// rasmx86_64 - src/shr/mem.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    conf::*,
    shr::{
        atype::{AType, ToAType},
        error::RASMError,
        kwd::Keyword,
        num::Number,
        reg::{Purpose as RPurpose, Register},
        size::Size,
    },
};
use std::str::FromStr;

impl Mem {
    pub fn try_make(memstr: &str, size_spec: Option<Keyword>) -> Result<Self, RASMError> {
        let size = if let Some(kwd) = size_spec {
            match Size::try_from(kwd){
                Ok(s) => s,
                Err(_) => return Err(RASMError::new(
                    None,
                    Some(format!("Invalid size specifier found `{}` in memory declaration", kwd.to_string())),
                    Some("Consider changing size specifier to either one: !qword, !dword, !word, !byte".to_string())
                ))
            }
        } else {
            return Err(RASMError::new(
                None,
                Some("No size specifier found in memory declaration".to_string()),
                Some("Consider adding size specifier after memory declaration like: !qword, !dword, !word or !byte".to_string())
            ));
        };
        mem_par(&mem_tok(memstr), size)
    }
    pub fn size(&self) -> Size {
        match self {
            Self::Index(_, _, size)
            | Self::IndexOffset(_, _, _, size)
            | Self::SIBOffset(_, _, _, _, size)
            | Self::SIB(_, _, _, size)
            | Self::Offset(_, _, size)
            | Self::Direct(_, size) => *size,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mem {
    // example: (%rax+20) !dword
    Offset(Register, i32, Size),
    // example: (%rax) !dword
    Direct(Register, Size),
    // example: (%rax * 2) !dword - uses scale and index, no base (SIB)
    Index(Register, Size, Size),
    IndexOffset(Register, i32, Size, Size),
    SIB(Register, Register, Size, Size),
    SIBOffset(Register, Register, Size, i32, Size),
}

#[derive(PartialEq, Debug, Clone)]
enum MemTok {
    Reg(Register),
    Num(i32),
    Unknown(String),
    Start,
    End,
    Plus,
    Minus,
    Star,
    Underline, // _
}

#[derive(Clone, Copy, PartialEq)]
enum NumberVariant {
    Multiply,
    Minus,
    Plus,
}

fn mem_par(toks: &[MemTok], size: Size) -> Result<Mem, RASMError> {
    if !toks.starts_with(&[MemTok::Start]) {
        return Err(RASMError::new(
            None,
            Some(format!(
                "Expected memory to start with: {}, found unexpected token",
                MEM_START
            )),
            Some(format!("Consider starting memory with {}", MEM_START)),
        ));
    }
    if !toks.ends_with(&[MemTok::End]) {
        return Err(RASMError::new(
            None,
            Some(format!("Expected memory closing symbol '{}'", MEM_CLOSE)),
            Some(format!(
                "Consider ending memory with '{}' character",
                MEM_CLOSE
            )),
        ));
    }

    let too_many_reg_e: RASMError = RASMError::new(
        None,
        Some("Too many registers found in one memory declaration".to_string()),
        Some("maximum amount of registers is 2".to_string()),
    );

    let mut variant: NumberVariant = NumberVariant::Plus;
    let mut tried_base: bool = false;
    let mut base: Option<Register> = None;
    let mut index: Option<Register> = None;
    let mut scale: Option<Size> = None;
    let mut offset: Option<i32> = None;
    for t in toks[1..toks.len() - 1].iter() {
        match t {
            MemTok::Reg(r) => {
                if tried_base {
                    if index.is_none() {
                        index = Some(*r);
                        tried_base = false;
                    } else {
                        return Err(too_many_reg_e);
                    }
                } else {
                    if base.is_none() {
                        base = Some(*r);
                    } else if index.is_none() {
                        index = Some(*r);
                    } else {
                        return Err(too_many_reg_e);
                    }
                }
            }
            MemTok::Underline => tried_base = true,
            MemTok::Num(n) => {
                if variant == NumberVariant::Multiply {
                    if let Ok(s) = Size::try_from(*n as u16) {
                        if scale.is_none() {
                            scale = Some(s);
                        } else {
                            return Err(RASMError::new(
                                None,
                                Some("Too many scales found in one memory declaration. Expected only 1 scale, found 2 (or more)".to_string()),
                                Some("Consider removing last scale (prefixed with '*')".to_string())
                            ));
                        }
                    } else {
                        return Err(RASMError::new(
                            None,
                            Some(format!("Found invalid'ly formatted scale: expected either 1, 2, 4 or 8, found {}", n)),
                            Some("Consider changing scale into number: 1, 2, 4 or 8".to_string())
                        ));
                    }
                } else {
                    if offset.is_none() {
                        if variant == NumberVariant::Minus {
                            offset = Some(-n);
                        } else {
                            offset = Some(*n);
                        }
                    } else if scale.is_none() {
                        if let Ok(s) = Size::try_from(*n as u16) {
                            scale = Some(s);
                        } else {
                            return Err(RASMError::new(
                                None,
                                Some(format!("Found invalid'ly formatted scale: expected either 1, 2, 4 or 8, found {}", n)),
                                Some("Consider changing scale into number: 1, 2, 4 or 8".to_string())
                            ));
                        }
                    } else {
                        return Err(RASMError::new(
                            None,
                            Some("Too many numbers found in one memory declaration. Expected max 2 numbers, found 3 (or more)".to_string()),
                            Some("Consider removing last scale (prefixed with '+' or '-')".to_string())
                        ));
                    }
                }

                variant = NumberVariant::Plus;
            }
            MemTok::Plus => variant = NumberVariant::Plus,
            MemTok::Minus => variant = NumberVariant::Minus,
            MemTok::Star => variant = NumberVariant::Multiply,
            MemTok::Unknown(s) => {
                return Err(RASMError::new(
                    None,
                    Some(format!(
                        "Found unknown token inside memory declaration: `{}`",
                        s
                    )),
                    Some(
                        "Consider changing this token into number, register, ',' or '_'"
                            .to_string(),
                    ),
                ))
            }
            MemTok::Start | MemTok::End => {
                return Err(RASMError::new(
                    None,
                    Some(
                        "Found memory closing/starting delimeter inside memory declaration"
                            .to_string(),
                    ),
                    Some(
                        "Consider removing closing/starting delimeter from memory declaration"
                            .to_string(),
                    ),
                ))
            }
        }
    }

    let mut rsize = Size::Any;
    if let Some(base) = base {
        if base.purpose() != RPurpose::General {
            return Err(RASMError::new(
                None,
                Some("Base register in this memory declaration isn't general purpose".to_string()),
                None,
            ));
        }
        rsize = base.size();
    }
    if let Some(index) = index {
        if index.purpose() != RPurpose::General {
            return Err(RASMError::new(
                None,
                Some("Index register in this memory declaration isn't general purpose".to_string()),
                None,
            ));
        }
        if index.size() != rsize {
            return Err(RASMError::new(
                None,
                Some(format!("Memory cannot be created, because one of registers is of invalid size: base size = {rsize}")),
                None
            ));
        }
    }

    match (base, index, scale, offset) {
        (Some(b), Some(i), Some(s), Some(o)) => Ok(Mem::SIBOffset(b, i, s, o, size)),
        (Some(b), Some(i), Some(s), None) => Ok(Mem::SIB(b, i, s, size)),
        (None, Some(b), None, Some(o)) | (Some(b), None, None, Some(o)) => {
            Ok(Mem::Offset(b, o, size))
        }
        (Some(b), None, None, None) => Ok(Mem::Direct(b, size)),
        (Some(b), Some(i), None, None) => Ok(Mem::SIB(b, i, Size::Byte, size)),
        (Some(i), None, Some(s), None) => Ok(Mem::Index(i, s, size)),
        (Some(i), None, Some(s), Some(o)) => Ok(Mem::IndexOffset(i, o, s, size)),
        (Some(b), Some(i), None, Some(o)) => Ok(Mem::SIBOffset(b, i, Size::Byte, o, size)),
        (None, None, None, None) => Err(RASMError::new(
            None,
            Some("Tried to make memory operand out of nothing `()`".to_string()),
            None,
        )),
        _ => Err(RASMError::new(
            None,
            Some(format!(
                "Unexpected memory combo: base = {:?}, index = {:?}, scale = {:?}, offset = {:?}",
                base, index, scale, offset
            )),
            None,
        )),
    }
}

fn mem_tok(str: &str) -> Vec<MemTok> {
    let mut pfx: Option<char> = None;
    let mut tmp_buf: Vec<char> = Vec::new();
    let mut tokens: Vec<MemTok> = Vec::new();
    for c in str.chars() {
        match (pfx, c) {
            (None, ' ' | '\t') => continue,
            (_, ',' | '+' | '-' | '*') => {
                if !tmp_buf.is_empty() {
                    if let Some(m) = mem_tok_make(&tmp_buf, pfx) {
                        tokens.push(m);
                    } else {
                        tokens.push(MemTok::Unknown(String::from_iter(tmp_buf.iter())));
                    }
                }
                pfx = None;
                tmp_buf = Vec::new();
                match c {
                    '+' => tokens.push(MemTok::Plus),
                    '-' => tokens.push(MemTok::Minus),
                    '*' => tokens.push(MemTok::Star),
                    _ => {}
                }
            }
            (_, '_') => tokens.push(MemTok::Underline),
            (_, MEM_START) => tokens.push(MemTok::Start),
            (_, MEM_CLOSE) => {
                if !tmp_buf.is_empty() {
                    if let Some(m) = mem_tok_make(&tmp_buf, pfx) {
                        tokens.push(m);
                    } else {
                        tokens.push(MemTok::Unknown(String::from_iter(tmp_buf.iter())));
                    }
                }
                tokens.push(MemTok::End)
            }
            (None, PREFIX_REG) => pfx = Some(PREFIX_REG),
            (None, PREFIX_VAL) => pfx = Some(PREFIX_VAL),
            (_, ' ') => continue,
            (_, _) => tmp_buf.push(c),
        }
    }
    tokens
}

fn mem_tok_make(tmp_buf: &[char], pfx: Option<char>) -> Option<MemTok> {
    let str = String::from_iter(tmp_buf.iter());
    match pfx {
        Some(PREFIX_REG) => res2op::<Register, (), MemTok>(Register::from_str(&str)),
        _ => {
            if let Ok(numb) = Number::from_str(&str) {
                match numb.get_int() {
                    Some(i) => res2op::<i32, _, MemTok>(i.try_into()),
                    None => {
                        if let Some(i) = numb.get_uint() {
                            res2op::<i32, _, MemTok>(i.try_into())
                        } else {
                            None
                        }
                    }
                }
            } else {
                None
            }
        }
    }
}

// allows me to save up to 4 lines...
// ... by adding 8 more lines (ok, maybe 4 lines are saved)
#[inline]
fn res2op<T, Y, E>(res: Result<T, Y>) -> Option<E>
where
    T: Into<E>,
{
    match res {
        Ok(t) => Some(t.into()),
        Err(_) => None,
    }
}

impl From<i32> for MemTok {
    fn from(num: i32) -> MemTok {
        MemTok::Num(num)
    }
}

impl From<Register> for MemTok {
    fn from(reg: Register) -> MemTok {
        MemTok::Reg(reg)
    }
}

impl ToAType for Mem {
    fn atype(&self) -> AType {
        AType::Memory(self.size())
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Mem{
    fn to_string(&self) -> String{
        match self{
            Self::Direct(bs, sz) => 
                format!("{PREFIX_KWD}{sz} {MEM_START}{PREFIX_REG}{}{MEM_CLOSE}", bs.to_string()),
            Self::Offset(bs,of,sz) => 
                format!("{PREFIX_KWD}{sz} {MEM_START}{PREFIX_REG}{}{}{MEM_CLOSE}", bs.to_string(),
                    if of <= &0{
                        format!("- {of}")
                    }
                    else {
                        format!("+ {of}")
                    }
                ),
            Self::Index(id, sc, sz) =>
                format!("{PREFIX_KWD}{sz} {MEM_START}{PREFIX_REG}{} * {sc}{MEM_CLOSE}", id.to_string()),
            Self::IndexOffset(id, of, sc, sz) =>
                format!("{PREFIX_KWD}{sz} {MEM_START}{PREFIX_REG}{} * {} {}{MEM_CLOSE}", id.to_string(),
                    <Size as Into<u8>>::into(*sc),
                    if of <= &0{
                        format!("- {of}")
                    }
                    else {
                        format!("+ {of}")
                    }
                ),
            Self::SIB(bs, id, sc, sz) => 
                format!("{PREFIX_KWD}{sz} {MEM_START}{PREFIX_REG}{} + {PREFIX_REG}{} * {}{MEM_CLOSE}", 
                    bs.to_string(), id.to_string(), <Size as Into<u8>>::into(*sc)),
            Self::SIBOffset(bs, id, sc, of, sz) => 
                format!("{PREFIX_KWD}{sz} {MEM_START}{PREFIX_REG}{} + {PREFIX_REG}{} * {} {}{MEM_CLOSE}", 
                    bs.to_string(), id.to_string(), <Size as Into<u8>>::into(*sc),
                    if of <= &0{
                        format!("- {of}")
                    }
                    else {
                        format!("+ {of}")
                    }
                ),
        }
    }
}

#[cfg(test)]
mod mem_test {
    use super::*;
    #[test]
    fn mem_tok_t() {
        assert!(
            vec![
                MemTok::Start,
                MemTok::Reg(Register::RAX),
                MemTok::Underline,
                MemTok::Num(20),
                MemTok::Num(20),
                MemTok::End,
            ] == mem_tok("(%rax, _, $20, 20)")
        )
    }
    #[test]
    fn mem_par_t() {
        let memtoks = vec![
            MemTok::Start,
            MemTok::Reg(Register::RAX),
            MemTok::Num(20),
            MemTok::End,
        ];
        assert!(Ok(Mem::Offset(Register::RAX, 20, Size::Dword)) == mem_par(&memtoks, Size::Dword));
        let memtoks = vec![
            MemTok::Start,
            MemTok::Underline,
            MemTok::Reg(Register::RAX),
            MemTok::Num(8),
            MemTok::End,
        ];
        assert!(Ok(Mem::Offset(Register::RAX, 8, Size::Byte)) == mem_par(&memtoks, Size::Byte));
    }
}
