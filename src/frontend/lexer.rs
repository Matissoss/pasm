//  rasmx86_64  -   lexer.rs
//  ------------------------
//  made by matissoss
//  licensed under MPL 2.0

use crate::frontend::{
    register::Register,
    FromToken,
    tokenizer::{
        Token,
        Value,
    },
};

pub struct Lexer;

#[derive(PartialEq)]
pub enum DValue{
    Register    (Register),
    MemAddr     (Register),
    BSSRef      (String),
    MemWOffset  (Register, i64)
}

#[derive(Default, PartialEq)]
pub struct Instruction{
    ins: String,
    src: Option<Value>,
    des: Option<DValue>
}

impl FromToken for Instruction{
    fn from_token(tokens: &[Token]) -> Option<Self>{
        let mut instruction = Instruction::default();
        if let Some(Token::String(ins)) = tokens.get(0){
            instruction.ins = ins.to_string();
        }
        if let Some(Token::Value(des)) = tokens.get(1) {
            instruction.des = match des{
                Value::Register(r)      => Some(DValue::Register(*r)),
                Value::BSSRef(r)        => Some(DValue::BSSRef(r.to_string())),
                Value::MemAddr(r)       => Some(DValue::MemAddr(*r)),
                Value::MemWOffset(r,o)  => Some(DValue::MemWOffset(*r,*o)),
                _ => None
            }
        }
        if let Some(Token::Value(src)) = tokens.get(2) {
            instruction.src = Some(src.clone());
        }
        if instruction != Instruction::default(){
            return Some(instruction);
        }
        else {
            return None;
        }
    }
}
