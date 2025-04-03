//  rasmx86_64  -  lex.rs
//  ----------------------
//  made by matissoss
//  licensed under MPL 2.0

use crate::{
    conf::PREFIX_KWD,
    color::ColorText,
    pre::tok::{
        Token,
        Tokens
    },
    shr::{
        mem::Mem,
        ast::{
            ASTNode,
            Operand,
            ASTInstruction,
            VarDec,
        },
        kwd::Keyword,
    }
};

#[derive(Debug, Clone)]
pub struct ErrContext{
    line_con: Vec<Token>,
    line_num: usize,
}

impl ErrContext{
    fn new(toks: &[Token], numb: usize) -> Self{
        return Self {
            line_con : toks.to_vec().clone(),
            line_num: numb
        };
    }
}

// All errors follow convention:
// 
// Name(ErrContext, Expected, Found): e.g. SynErr::UnexpectedType()
// Name(ErrContext, UnexpectedValue): e.g. LogErr::InvalidDst()
// Name(ErrContext)                 : e.g. LogErr::NoDst()
#[derive(Debug, Clone)]
pub enum SynErr{
    UnexpectedToken(ErrContext, String, String),
    TooManyArgs(ErrContext, usize, usize),
    UnexpectedEOL(ErrContext, String),
}

#[derive(Debug, Clone)]
pub enum LexErr{
    Syntax(SynErr),
    Other,
    Unknown(ErrContext, String)
}

pub struct Lexer;
impl Lexer{
    pub fn parse_file(file: Vec<Vec<Token>>) -> Vec<Result<(ASTNode, usize), LexErr>>{
        let mut line_count : usize = 0;
        let mut ast_tree   : Vec<Result<(ASTNode, usize), LexErr>> = Vec::new();
        for line in file {
            line_count += 1;
            if line.is_empty(){
                continue;
            }
            
            let mut node  : Option<ASTNode> = None;
            let mut error : Option<LexErr>  = None;

            match line.get(0){
                Some(Token::Label(lbl)) => node = Some(ASTNode::Label(lbl.to_string())),
                Some(Token::Keyword(Keyword::End)) => node = Some(ASTNode::End),
                Some(Token::Section(sec)) => node = Some(ASTNode::Section(sec.to_string())),
                Some(Token::Unknown(var)) => {
                    let size : u8 = match line.get(1) {
                        Some(Token::Keyword(Keyword::Qword)) => 8,
                        Some(Token::Keyword(Keyword::Dword)) => 4,
                        Some(Token::Keyword(Keyword::Word))  => 2,
                        Some(Token::Keyword(Keyword::Byte))  => 1,
                        _   => {
                            ast_tree.push(Err(LexErr::Unknown(
                                ErrContext::new(&line, line_count),
                                format!("Unexpected start of line!")
                            )));
                            continue;
                        }
                    };

                    let mut rest = line[2..].iter();
                    let mut cont = String::new();
                    
                    while let Some(e) = rest.next(){
                        if let Token::Immediate(n) = e{
                            if line[2..].len() != 1{
                                cont.push((*n as u64 & 0xFF) as u8 as char);
                            }
                            else{
                                cont.push_str(&n.to_string());
                            }
                        }
                        else if let Token::Comma = e {continue}
                        else if let Token::String(s) = e {
                            cont.push_str(&s);
                        }
                        else if let Token::Unknown(s) = e {
                            cont.push_str(&s);
                        }
                        else{
                            error = Some(LexErr::Unknown(
                                ErrContext::new(&line, line_count),
                                format!("unknown token {:?} found in variable declaration",e) 
                            ));
                        }
                    }
                    if let None = error{
                        node = Some(ASTNode::VarDec(VarDec{
                            name: var.to_string(),
                            size,
                            content: cont
                        }));
                    }
                },
                Some(Token::Keyword(Keyword::Global)) => {
                    if let Some(glob) = line.get(1){
                        node = Some(ASTNode::Global(glob.to_string()));
                    }
                    else {
                        error = Some(LexErr::Syntax(SynErr::UnexpectedEOL(
                                    ErrContext::new(&line, line_count),
                                    format!("something to be global")
                        )));
                    }
                }
                Some(Token::Instruction(ins)) => {
                    if let Some((dst_raw, src_raw)) = vec_split_by(&line, Token::Comma){
                        let inst_dst = make_op(&dst_raw[1..], (&line, line_count)).ok();
                        let inst_src = make_op(&src_raw[1..], (&line, line_count)).ok();

                        node = Some(ASTNode::Ins(ASTInstruction {ins: *ins, dst: inst_dst, src: inst_src, lin: line_count}));
                    }
                    else {
                        let operand = make_op(&line[1..], (&line, line_count)).ok();
                        node = Some(ASTNode::Ins(ASTInstruction {ins: *ins, dst: operand, src: None, lin: line_count}));
                    }
                },
                s => {
                    ast_tree.push(Err(LexErr::Unknown(
                        ErrContext::new(&line, line_count),
                        format!("Unexpected start of line: {:?}", s.unwrap())
                    )))
                }
            }

            if let Some(node_t) = node{
                ast_tree.push(Ok((node_t, line_count)));
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
            LexErr::Other => {
                format!("{}:\n\tidk where, try being better coder :)", "error".red())
            },
            LexErr::Unknown(cont, val) => {
                format!("{}:\n\tAt line {}:\n\t{}\n\t---\n\t{}", 
                    "error".red(), cont.line_num.to_string().as_str().bold_yellow(), 
                    Tokens(cont.line_con.clone()).to_string().as_str().green(), val)
            }
            LexErr::Syntax(serr)  => {
                match serr{
                    SynErr::UnexpectedEOL(context, expected) => {
                        format!("{}:\n\tAt line {}:\n\t{}\n\t---\n\tUnexpected end of line!\n\t{}\n\t\n\t{}", 
                            "error".red(),
                            context.line_num.to_string().as_str().bold_yellow(), 
                            Tokens(context.line_con.clone()).to_string().as_str().green(),
                            expected,
                            "tip: maybe you forgot to add something?")
                    }
                    SynErr::UnexpectedToken(context, expected, found) => {
                        format!("{}:\n\tAt line {}:\n\t{}\n\t---\n\tUnexpected token was found: expected {}, found {}"
                            , 
                            "error".red(),
                            context.line_num.to_string().as_str().bold_yellow(), 
                            Tokens(context.line_con.clone()).to_string().as_str().green(),
                            expected, found
                        )
                    },
                    SynErr::TooManyArgs(context, expected, found) => {
                        format!("{}:\n\tAt line {}:\n\t{}\n\t---\n\tToo many arguments found in instruction!\n\tExpected <= {}\n\tFound = {}\n\t\n\t{}", 
                            "error".red(),
                            context.line_num.to_string().as_str().bold_yellow(), 
                            Tokens(context.line_con.clone()).to_string().as_str().green(),
                            expected, found,
                            "tip: did you forgot about comma beetwen operands? or maybe you forgot that offset has to be placed without space in memory address?"
                        )
                    },
                }
            }
        }
    }
}


fn vec_split_by<T>(vector: &Vec<T>, split_by: T) -> Option<(Vec<T>, Vec<T>)>
where T: PartialEq + Clone{
    let mut first_part : Vec<T> = Vec::new();
    let mut second_part : Vec<T> = Vec::new();
    
    let mut f_bool : bool = true;
    for element in vector{
        if *element == split_by{
            f_bool = false;
        }
        if f_bool{
            first_part.push(element.clone());
        }
        else{
            second_part.push(element.clone());
        }
    }
    if first_part.is_empty() == false && second_part.is_empty() == false{
        return Some((first_part, second_part));
    }
    else {
        return None;
    }
}

fn make_op(tok: &[Token], cont: (&[Token], usize)) -> Result<Operand, LexErr>{
    match tok.len(){
        1 => {
            Operand::try_from(tok[0].clone()).map_err(|_| LexErr::Other)
        },
        2 => {
            if let (Token::MemAddr(mem), Token::Keyword(kwd)) = (&tok[0], &tok[1]){
                if let Some(memaddr) = Mem::create(&mem, Some(*kwd)){
                    Ok(Operand::Mem(memaddr))
                }
                else {
                    Err(LexErr::Syntax(SynErr::UnexpectedToken(
                            ErrContext::new(cont.0, cont.1),
                            format!("memory address and size specificator, either one [!dword, !word, !byte, !qword]"),
                            format!("{} {}{}", mem, PREFIX_KWD, kwd.to_string())        
                    )))
                }
            }
            else {
                Err(LexErr::Other)
            }
        },
        l => Err(LexErr::Syntax(SynErr::TooManyArgs(
            ErrContext::new(cont.0, cont.1),2,l
        )))
    }
}
