//  rasmx86_64  -  lex.rs
//  ----------------------
//  made by matissoss
//  licensed under MPL 2.0

#[allow(unused)]
use crate::{
    conf::FAST_MODE,
    color::ColorText,
    pre::tok::{
        Token,
        Tokens
    },
    shr::{
        reg::Register,
        mem::Mem,
        ast::{
            ASTNode,
            Operand,
            AstInstruction,
            AsmType,
            VarDec,
            AsmTypes,
        },
        kwd::Keyword,
        ins::Instruction,
    }
};

#[allow(unused)]
#[derive(Debug)]
pub struct ErrContext{
    line_con: Vec<Token>,
    line_num: usize,
}

// All errors follow convention:
// 
// Name(ErrContext, Expected, Found): e.g. SynErr::UnexpectedType()
// Name(ErrContext, UnexpectedValue): e.g. LogErr::InvalidDst()
// Name(ErrContext)                 : e.g. LogErr::NoDst()
#[allow(unused)]
#[derive(Debug)]
pub enum LogErr{
    InvalidDst(ErrContext, Vec<AsmType>, Token),
    InvalidSrc(ErrContext, Vec<AsmType>, Token),
    InvalidOpr(ErrContext, Token, Token),
    TooLarge  (ErrContext, usize, usize)
}
#[allow(unused)]
#[derive(Debug)]
pub enum SynErr{
    UnexpectedToken(ErrContext, String, String),
    UnexpectedType (ErrContext, Vec<AsmType>, AsmType),
    InstructionTooShort(ErrContext, usize, usize)
}

#[allow(unused)]
#[derive(Debug)]
pub enum LexErr{
    Logical(LogErr),
    Syntax(SynErr),
}

pub struct Lexer;
impl Lexer{
    pub fn parse_file(file: Vec<Vec<Token>>) -> Vec<Result<ASTNode, LexErr>>{
        let mut line_count : usize = 0;
        let mut ast_tree   : Vec<Result<ASTNode, LexErr>> = Vec::new();
        for line in file {
            line_count += 1;
            if line.is_empty(){
                continue;
            }
            
            let mut node  : Option<ASTNode> = None;
            let mut error : Option<LexErr>  = None;




            if let Some(node_t) = node{
                ast_tree.push(Ok(node_t));
            }
            else if let Some(error_t) = error{
                ast_tree.push(Err(error_t));
            }

        }
        return ast_tree;
    }
}

impl ToString for LexErr{
    fn to_string(&self) -> String{
        match self{
            LexErr::Logical(lerr) => {
                match lerr{
                    LogErr::InvalidSrc(context, expected, found) => {
                        format!("{}:\n\tAt line {}:\n\t`{}`\n\t---\n\tExpected: {}\n\tFound: {} in src field (2 operand)", "error".red(), 
                            context.line_num.to_string().as_str().bold_yellow(), Tokens(context.line_con.clone()).to_string(),
                            AsmTypes(expected.to_vec().clone()).to_string(), found.to_string()
                        )
                    },
                    LogErr::InvalidDst(context, expected, found) => {
                        format!("{}:\n\tAt line {}:\n\t`{}`\n\t---\n\tExpected: {}\n\tFound: `{}` in dst field (1 operand)", "error".red(), 
                            context.line_num.to_string().as_str().bold_yellow(), Tokens(context.line_con.clone()).to_string(),
                            AsmTypes(expected.to_vec().clone()).to_string(), found.to_string()
                        )
                    },
                    LogErr::TooLarge(context, expected, _) => {
                        format!("{}:\n\tAt line {}: `{}`\n\t---\n\tExpected number {}-bit, found larger number!", "error".red(),
                            context.line_num.to_string().as_str().bold_yellow(), Tokens(context.line_con.clone()).to_string(),
                            expected * 8,
                        )
                    },
                    LogErr::InvalidOpr(context, _, _) => {
                        format!("{}:\n\tAt line {}: `{}`\n\t---\n\tInvalid operands were found in source and destination", "error".red(),
                        context.line_num.to_string().as_str().bold_yellow(), Tokens(context.line_con.clone()).to_string())
                    }
                }
            },
            LexErr::Syntax(serr)  => {
                match serr{
                    SynErr::UnexpectedToken(context, expected, found) => {
                        format!("{}:\n\tAt line {}: `{}`\n\t---\n\tUnexpected token was found: expected {}, found {}", "error".red(),
                            context.line_num.to_string().as_str().bold_yellow(), Tokens(context.line_con.clone()).to_string(),
                            expected, found
                        )
                    },
                    SynErr::UnexpectedType(context, expected, found) => {
                        format!("{}:\n\tAt line {}: `{}`\n\t---\n\tUnexpected type was found: expected {}, found {}", "error".red(),
                            context.line_num.to_string().as_str().bold_yellow(), Tokens(context.line_con.clone()).to_string(),
                            AsmTypes(expected.to_vec().clone()).to_string(), found.to_string()
                        )
                    },
                    SynErr::InstructionTooShort(context, _, _) => {
                        format!("{}:\n\tAt line {}: `{}`\n\t---\n\tUnexpected end of instruction!", "error".red(),
                            context.line_num.to_string().as_str().bold_yellow(), Tokens(context.line_con.clone()).to_string(),
                        )
                    }
                }
            }
        }
    }
}
