//  rasmx86_64  -  lex.rs
//  ----------------------
//  made by matissoss
//  licensed under MPL 2.0

use crate::{
    pre::tok::Token,
    shr::{
        reg::Register,
        kwd::Keyword,
    },
    conf::FAST_MODE
};

pub struct Lexer;

enum MemType{
    Dword,
    Qword,
     Word,
     Byte
}

enum LexerValue{
    Register(Register),
    Mem(MemType, Register),
    MemOffset(MemType, Register, i64),
    SIB(MemType, Register, Register, i64),

}

impl Lexer {
    fn lexr(tokens: Vec<Vec<Token>>) -> Vec<LexerValue>{
        Vec::new()
    }
}
