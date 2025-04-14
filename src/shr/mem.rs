// rasmx86_64 - mem.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use std::str::FromStr;
use crate::{
    shr::{
        reg::Register,
        kwd::Keyword,
        num::Number,
        error::{
            RASMError,
            ExceptionType as ExType
        },
        size::Size,
        atype::{
            AType,
            ToAType
        }
    },
    conf::{
        PREFIX_REG,
        PREFIX_VAL,
        MEM_CLOSE,
        MEM_START,
    }
};

impl Mem {
    pub fn new(memstr: &str, size_spec: Option<Keyword>) -> Result<Self, RASMError>{
        let size = if let Some(kwd) = size_spec{
            match kwd {
                Keyword::Qword => Size::Qword,
                Keyword::Dword => Size::Dword,
                Keyword::Word  => Size::Word,
                Keyword::Byte  => Size::Byte,
                _ => return Err(RASMError::new(
                    None,
                    ExType::Error,
                    None,
                    Some(format!("Invalid size specifier found `{}` in memory declaration", kwd.to_string())),
                    Some(format!("Consider changing size specifier to either one: !qword, !dword, !word, !byte"))
                ))
            }
        } else {
            return Err(RASMError::new(
                None,
                ExType::Error,
                None,
                Some(format!("No size specifier found in memory declaration")),
                Some(format!("Consider adding size specifier after memory declaration like: !qword, !dword, !word or !byte"))
            ))
        };
        mem_par(&mem_tok(memstr), size)
    }
    pub fn size(&self) -> Size{
        match self{
            Self::Index(_, _, size) |Self::IndexOffset(_, _, _, size)| Self::RipRelative(_, size)|
            Self::SIBOffset(_, _, _, _, size)|
            Self::SIB(_,_,_,size)|Self::Offset(_,_,size)|Self::Direct(_, size) => *size,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mem{
    // example: (%rax+20) !dword
    Offset(Register, i32, Size),
    // example: (%rax) !dword
    Direct(Register, Size),
    // example: (%rax * 2) !dword - uses scale and index, no base (SIB)
    Index(Register, Size, Size),
    IndexOffset(Register, i32, Size, Size),
    // example: (%rip+20) !dword
    RipRelative(i32, Size),

    SIB(Register, Register, Size, Size),
    SIBOffset(Register, Register, Size, i32, Size),
}

#[derive(PartialEq, Debug, Clone)]
enum MemTok{
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
enum NumberVariant{
    Multiply,
    Minus,
    Plus,
}

fn mem_par(toks: &[MemTok], size: Size) -> Result<Mem, RASMError>{
    if !toks.starts_with(&[MemTok::Start]){
        return Err(RASMError::new(
            None,
            ExType::Error,
            None,
            Some(format!("Expected memory to start with: {}, found unexpected token", MEM_START)),
            Some(format!("Consider starting memory with {}", MEM_START))
        ))
    }
    if !toks.ends_with(&[MemTok::End]){
        return Err(RASMError::new(
            None,
            ExType::Error,
            None,
            Some(format!("Expected memory closing symbol '{}'", MEM_CLOSE)),
            Some(format!("Consider ending memory with '{}' character", MEM_CLOSE)),
        ))
    }

    let too_many_reg_e : RASMError = RASMError::new(
        None,
        ExType::Error,
        None,
        Some(format!("Too many registers found in one memory declaration")),
        Some(format!("maximum amount of registers is 2"))
    );

    let mut rest                            = toks[1..toks.len()-1].iter();
    let mut variant     : NumberVariant     = NumberVariant::Plus;
    let mut tried_base  : bool              = false;
    let mut base        : Option<Register>  = None;
    let mut index       : Option<Register>  = None;
    let mut scale       : Option<Size>      = None;
    let mut offset      : Option<i32>       = None;
    while let Some(t) = rest.next(){
        match t {
            MemTok::Reg(r) => {
                if tried_base{
                    if let None = index {
                        index = Some(*r);
                        tried_base = false;
                    }
                    else {
                        return Err(too_many_reg_e)
                    }
                }
                else {
                    if let None = base {
                        base = Some(*r);
                    }
                    else if let None = index {
                        index = Some(*r);
                    }
                    else {
                        return Err(too_many_reg_e)
                    }
                }
            },
            MemTok::Underline => tried_base = true,
            MemTok::Num(n) => {
                if variant == NumberVariant::Multiply{
                    if let Ok(s) = Size::try_from(*n as u16){
                        if let None = scale{
                            scale = Some(s);
                        }
                        else {
                            return Err(RASMError::new(
                                None,
                                ExType::Error,
                                None,
                                Some(format!("Too many scales found in one memory declaration. Expected only 1 scale, found 2 (or more)")),
                                Some(format!("Consider removing last scale (prefixed with '*')"))
                            ))
                        }
                    }
                    else {
                        return Err(RASMError::new(
                            None,
                            ExType::Error,
                            None,
                            Some(format!("Found invalid'ly formatted scale: expected either 1, 2, 4 or 8, found {}", n)),
                            Some(format!("Consider changing scale into number: 1, 2, 4 or 8"))
                        ))
                    }
                }
                else {
                    if let None = offset {
                        offset = Some(*n);
                    }
                    else if let None = scale{
                        if let Ok(s) = Size::try_from(*n as u16){
                            scale = Some(s);
                        }
                        else {
                            return Err(RASMError::new(
                                None,
                                ExType::Error,
                                None,
                                Some(format!("Found invalid'ly formatted scale: expected either 1, 2, 4 or 8, found {}", n)),
                                Some(format!("Consider changing scale into number: 1, 2, 4 or 8"))
                            ))
                        }
                    }
                    else {
                        return Err(RASMError::new(
                            None,
                            ExType::Error,
                            None,
                            Some(format!("Too many numbers found in one memory declaration. Expected max 2 numbers, found 3 (or more)")),
                            Some(format!("Consider removing last scale (prefixed with '+' or '-')"))
                        ))
                    }
                }

                variant = NumberVariant::Plus;
            },
            MemTok::Plus     => variant = NumberVariant::Plus,
            MemTok::Minus    => variant = NumberVariant::Minus,
            MemTok::Star     => variant = NumberVariant::Multiply,
            MemTok::Unknown(s) => return Err(RASMError::new(
                None,
                ExType::Error,
                None,
                Some(format!("Found unknown token inside memory declaration: `{}`", s)),
                Some(format!("Consider changing this token into number, register, ',' or '_'"))
            )),
            MemTok::Start|MemTok::End => return Err(RASMError::new(
                None,
                ExType::Error,
                None,
                Some(format!("Found memory closing/starting delimeter inside memory declaration")),
                Some(format!("Consider removing closing/starting delimeter from memory declaration"))
            )),
        }
    }

    match (base, index, scale, offset){
        (Some(b), Some(i), Some(s), Some(o))        => Ok(Mem::SIBOffset(b, i, s, o, size)),
        (Some(b), Some(i), Some(s), None)           => Ok(Mem::SIB(b, i, s, size)),
        (Some(Register::RIP), None, None, Some(o))  => Ok(Mem::RipRelative(o, size)),
        (Some(b), None   , None   , Some(o))        => Ok(Mem::Offset(b, o, size)),
        (Some(b), None   , None   , None   )        => Ok(Mem::Direct(b, size)),
        (Some(b), Some(i), None   , None   )        => Ok(Mem::SIB(b, i, Size::Byte, size)),
        (Some(i), None   , Some(s), None   )        => Ok(Mem::Index(i, s, size)),
        (Some(i), None   , Some(s), Some(o))        => Ok(Mem::IndexOffset(i, o, s, size)),
        (Some(_), Some(_), None, Some(_))           => Err(RASMError::new(None, ExType::Error, None,
            Some(format!("Tried to use SIB memory addresation, but scale was not found")),
            Some(format!("Consider adding scale. Scale can be added like this: `*<scale>`, 
             where <scale> is number 1, 2, 4 or 8. 
             If you had started scale with comma ',' instead of asterisk '*',
             it got handled as displacement and not scale."))
        )),
        _ => {
            println!("{:?} {:?} {:?} {:?}", base, index, scale, offset);
            panic!("Unexpected memory combo :)")
        }
    }
}

fn mem_tok(str: &str) -> Vec<MemTok>{
    let mut pfx : Option<char> = None;
    let mut tmp_buf : Vec<char> = Vec::new();
    let mut tokens : Vec<MemTok> = Vec::new();
    for c in str.chars(){
        match (pfx, c) {
            (None, ' '|'\t') => continue,
            (_, ','|'+'|'-'|'*') => {
                if !tmp_buf.is_empty(){
                    if let Some(m) = mem_tok_make(&tmp_buf, pfx){
                        tokens.push(m);
                    }
                    else {
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
            },
            (_, '_') => tokens.push(MemTok::Underline),
            (_, MEM_START) => tokens.push(MemTok::Start),
            (_, MEM_CLOSE) => {
                if !tmp_buf.is_empty(){
                    if let Some(m) = mem_tok_make(&tmp_buf, pfx){
                        tokens.push(m);
                    }
                    else {
                        tokens.push(MemTok::Unknown(String::from_iter(tmp_buf.iter())));
                    }
                }
                tokens.push(MemTok::End)
            },
            (None, PREFIX_REG) => pfx = Some(PREFIX_REG),
            (None, PREFIX_VAL) => pfx = Some(PREFIX_VAL),
            (_, _) => tmp_buf.push(c),
        }
    }
    return tokens;
}

fn mem_tok_make(tmp_buf : &[char], pfx: Option<char>) -> Option<MemTok>{
    let str = String::from_iter(tmp_buf.iter());
    match pfx{
        Some(PREFIX_REG) => res2op::<Register, (), MemTok>(Register::from_str(&str)),
        _ => if let Ok(numb) = Number::from_str(&str){
            match numb.get_int(){
                Some(i) => {
                    return res2op::<i32, _, MemTok>(i.try_into());
                },
                None => {
                    if let Some(i) = numb.get_uint(){
                        return res2op::<i32, _, MemTok>(i.try_into());
                    }
                    else {
                        None
                    }
                }
            }
        }
        else {
            None
        },
    }
}


// allows me to save up to 4 lines
#[inline]
fn res2op<T, Y, E>(res: Result<T, Y>) -> Option<E>
where T: Into<E>{
    match res{
        Ok(t) => Some(t.into()),
        Err(_) => None
    }
}

impl Into<MemTok> for i32{
    fn into(self) -> MemTok{
        return MemTok::Num(self)
    }
}
impl Into<MemTok> for Register{
    fn into(self) -> MemTok{
        return MemTok::Reg(self)
    }
}

impl ToString for MemTok{
    fn to_string(&self) -> String{
        match self {
            Self::Reg(r)        => r.to_string(),
            Self::End           => MEM_CLOSE.to_string(),
            Self::Start         => MEM_START.to_string(),
            Self::Num(n)        => n.to_string(),
            Self::Underline     => '_'.to_string(),
            Self::Unknown(s)    => s.to_string(),
            Self::Star          => '*'.to_string(),
            Self::Minus         => '-'.to_string(),
            Self::Plus          => '+'.to_string()
        }
    }
}

impl ToAType for Mem{
    fn atype(&self) -> AType{
        return AType::Mem(self.size())
    }
}


#[cfg(test)]
mod mem_test{
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
    fn mem_par_t(){
        let memtoks = vec![
            MemTok::Start,
            MemTok::Reg(Register::RAX),
            MemTok::Num(20),
            MemTok::End,
        ];
        assert!(Ok(Mem::Offset(Register::RAX, 20, Size::Dword)) == mem_par(&memtoks, Size::Dword));
    }
}
