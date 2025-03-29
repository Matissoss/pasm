//  rasmx86_64  -  lex.rs
//  ----------------------
//  made by matissoss
//  licensed under MPL 2.0

use crate::{
    conf::FAST_MODE,
    pre::tok::Token,
    shr::{
        reg::Register,
        mem::Mem,
        ast::{
            ASTNode,
            Operand,
            AstInstruction,
            AsmType,
            VarDec,
        },
        kwd::Keyword,
        ins::Instruction,
    }
};

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
#[derive(Debug)]
pub enum LogErr{
    InvalidSize(ErrContext),
    NoDst(ErrContext),
    NoSrc(ErrContext),
    NoOpr(ErrContext),
    InvalidDst(ErrContext, Token),
    InvalidSrc(ErrContext, Token),
    InvalidOpr(ErrContext, Token, Token),
    TooLarge  (ErrContext, usize, usize)
}
#[derive(Debug)]
pub enum SynErr{
    UnexpectedToken(ErrContext, String, String),
    UnexpectedType (ErrContext, Vec<AsmType>, AsmType),
    InstructionTooShort(ErrContext, usize, usize)
}

#[derive(Debug)]
pub enum LexErr{
    Logical(LogErr),
    Syntax(SynErr),
    Other(String)
}

pub struct Lexer;
impl Lexer{
    pub fn parse_file(file: Vec<Vec<Token>>) -> Vec<Result<ASTNode, LexErr>>{
        let mut line_count : usize = 1;
        let mut ast_tree   : Vec<Result<ASTNode, LexErr>> = Vec::new();
        for line in file {
            if line.is_empty(){
                continue;
            }

            let mut node  : Option<ASTNode> = None;
            let mut error : Option<LexErr>  = None;
           
            match line.len(){
                0 => continue,
                1 => {
                    if let Token::Label(lbl) = line[0].clone() {
                        node = Some(ASTNode::Label(lbl));
                    }
                    else if let Token::Instruction(ins) = line[0]{
                        node = Some(ASTNode::Ins(AstInstruction{ins, dst: None, src: None}));
                    }
                },
                2 => {
                    if let Token::Keyword(Keyword::Section) = &line[0]{
                        if let Token::Unknown(name) = &line[1]{
                            node = Some(ASTNode::Section(name.to_string()));
                        }
                    }
                    else if let Token::Keyword(Keyword::Global) = &line[0]{
                        if let Token::Unknown(name) = &line[1]{
                            node = Some(ASTNode::Global(name.to_string()));
                        }
                    }
                    else if let Token::Instruction(ins) = line[0]{
                        if let Ok(dst) = Operand::try_from(line[1].clone()){
                            node = Some(ASTNode::Ins(AstInstruction{ins, dst: Some(dst), src: None}));
                        }
                    }
                },
                3 => {
                    if let Token::Instruction(ins) = line[0]{
                        if let (Ok(dst), Ok(src)) = (Operand::try_from(line[1].clone()), Operand::try_from(line[2].clone())){
                            node = Some(ASTNode::Ins(AstInstruction {ins, dst: Some(dst), src: Some(src)}));
                        }
                        else {
                            match (Operand::try_from(line[1].clone()), Operand::try_from(line[2].clone())){
                                (Err(_), Err(_)) => {
                                    error = Some(LexErr::Logical(LogErr::InvalidOpr(
                                        ErrContext {line_con: line.clone(),line_num: line_count},
                                        line[1].clone(),
                                        line[2].clone()
                                    )))
                                },
                                (Ok(_), Err(_)) => {
                                    error = Some(LexErr::Logical(LogErr::InvalidSrc(
                                        ErrContext {line_con: line.clone(),line_num: line_count},
                                                    line[2].clone(),
                                    )))
                                },
                                (Err(_), Ok(_)) => {
                                    error = Some(LexErr::Logical(LogErr::InvalidDst(
                                        ErrContext {line_con: line.clone(),line_num: line_count},
                                        line[1].clone(),
                                    )))
                                },
                                _ =>{}
                            }
                        }
                    }
                },
                _ => {
                    if let Token::Keyword(Keyword::Var) = line[0].clone(){
                        if let Token::Unknown(name) = line[1].clone(){
                            match line[2]{
                                 Token::Keyword(Keyword::Dd)
                                |Token::Keyword(Keyword::Dq)
                                |Token::Keyword(Keyword::Db)
                                |Token::Keyword(Keyword::Dw) => {
                                    let size = match line[2]{
                                        Token::Keyword(Keyword::Db) => 1,
                                        Token::Keyword(Keyword::Dw) => 2,
                                        Token::Keyword(Keyword::Dd) => 4,
                                        Token::Keyword(Keyword::Dq) => 8,
                                        _ => break
                                    };
                                    match line.get(3) {
                                        Some(Token::String(_)) => {
                                            let mut line_iter = line[3..].iter();

                                            let mut content = String::new();
                                            while let Some(st) = line_iter.next(){
                                                if let Token::Immediate(val) = st{
                                                    content.push(((*val as u64) & 0xFF) as u8 as char);
                                                }
                                                else if let Token::Unknown(val) = st{
                                                    content.push_str(val);
                                                }
                                                else if let Token::String(val) = st{
                                                    content.push_str(val);
                                                }
                                            }
                                            node = Some(ASTNode::VarDec(VarDec {
                                                name,
                                                bss: false,
                                                size,
                                                content
                                            }));
                                        },
                                        Some(Token::Immediate(val)) => {
                                            match size {
                                                1 => {
                                                    let vali8 : Result<i8, ()> = (*val).try_into().map_err(|_| ());
                                                    if let Err(()) = vali8{
                                                        let valu8 : Result<u8, ()> = (*val).try_into().map_err(|_| ());
                                                        if let Err(()) = valu8{
                                                            error = Some(LexErr::Logical(LogErr::TooLarge(
                                                                ErrContext {line_num: line_count, line_con: line.clone()},
                                                                size.into(),
                                                                8
                                                            )));
                                                        }
                                                    }
                                                },
                                                2 => {
                                                    let vali16 : Result<i16, ()> = (*val).try_into().map_err(|_| ());
                                                    if let Err(()) = vali16{
                                                        let valu16 : Result<u16, ()> = (*val).try_into().map_err(|_| ());
                                                        if let Err(()) = valu16{
                                                            error = Some(LexErr::Logical(LogErr::TooLarge(
                                                                ErrContext {line_num: line_count, line_con: line.clone()},
                                                                size.into(),
                                                                8
                                                            )));
                                                        }
                                                    }
                                                },
                                                4 => {
                                                    let vali32 : Result<i32, ()> = (*val).try_into().map_err(|_| ());
                                                    if let Err(()) = vali32{
                                                        let valu32 : Result<u32, ()> = (*val).try_into().map_err(|_| ());
                                                        if let Err(()) = valu32{
                                                            error = Some(LexErr::Logical(LogErr::TooLarge(
                                                                ErrContext {line_num: line_count, line_con: line.clone()},
                                                                size.into(),
                                                                8
                                                            )));
                                                        }
                                                    }
                                                },
                                                _ => {}
                                            }
                                            if let None = error{
                                                node = Some(ASTNode::VarDec(VarDec{
                                                    name,
                                                    bss: false,
                                                    size,
                                                    content: val.to_string()
                                                }));
                                            }
                                        },
                                        Some(Token::Register(_)) => {
                                            error = Some(LexErr::Syntax(SynErr::UnexpectedType(
                                                ErrContext {line_num: line_count, line_con: line.clone()},
                                                vec![AsmType::Imm,AsmType::ConstString],
                                                AsmType::Reg
                                            )));
                                        }
                                        Some(Token::MemAddr(_)) => {
                                            error = Some(LexErr::Syntax(SynErr::UnexpectedType(
                                                ErrContext {line_num: line_count, line_con: line.clone()},
                                                vec![AsmType::Imm,AsmType::ConstString],
                                                AsmType::Mem
                                            )));
                                        }
                                        None => {
                                            error = Some(LexErr::Syntax(SynErr::InstructionTooShort(
                                                ErrContext {line_num: line_count, line_con: line.clone()},
                                                4,
                                                line.len()
                                            )));
                                        },
                                        _ => {
                                            error = Some(LexErr::Syntax(SynErr::UnexpectedToken(
                                                ErrContext {line_num: line_count, line_con: line.clone()},
                                                format!("{:?}", line[3].clone()),
                                                format!("either one: [(comptime string), immX]")
                                            )))
                                        }
                                    }
                                },
                                _ => {
                                    error = Some(LexErr::Syntax(SynErr::UnexpectedToken(
                                                ErrContext {line_num: line_count, line_con: line.clone()},
                                                format!("{:?}", line[2]),
                                                format!("either one: [db, dw, dq, dd]")
                                    )));
                                }
                            }
                        }
                    }
                }
            }
            if let Some(node_t) = node{
                ast_tree.push(Ok(node_t));
            }
            else if let Some(error_t) = error{
                ast_tree.push(Err(error_t));
            }

            line_count += 1;
        }
        return ast_tree;
    }
}
