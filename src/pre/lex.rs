//  rasmx86_64  -  lex.rs
//  ----------------------
//  made by matissoss
//  licensed under MPL 2.0

use crate::{
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
        num::Number,
        kwd::Keyword,
        error::{
            RASMError,
            ExceptionType as ExType
        },
    }
};

pub struct Lexer;
impl Lexer{
    pub fn parse_file(file: Vec<Vec<Token>>) -> Vec<Result<(ASTNode, usize), RASMError>>{
        let mut line_count : usize = 0;
        let mut ast_tree   : Vec<Result<(ASTNode, usize), RASMError>> = Vec::new();
        for line in file {
            line_count += 1;
            if line.is_empty(){
                continue;
            }
            
            let mut node  : Option<ASTNode> = None;
            let mut error : Option<RASMError>  = None;

            match line.get(0){
                Some(Token::Label(lbl))             => node = Some(ASTNode::Label(lbl.to_string())),
                Some(Token::Keyword(Keyword::End))  => node = Some(ASTNode::End),
                Some(Token::Section(sec))           => node = Some(ASTNode::Section(sec.to_string())),
                Some(Token::Unknown(var))           => {
                    let size : u8 = match line.get(1) {
                        Some(Token::Keyword(Keyword::Qword)) => 8,
                        Some(Token::Keyword(Keyword::Dword)) => 4,
                        Some(Token::Keyword(Keyword::Word))  => 2,
                        Some(Token::Keyword(Keyword::Byte))  => 1,
                        _   => {
                            ast_tree.push(Err(RASMError::new(
                                Some(line_count),
                                ExType::Error,
                                Some(Tokens(line).to_string()),
                                Some(format!("Found wrong size specifier for memory!")),
                                Some(format!("Place either one of these after memory declaration: !byte, !word, !dword, !qword"))
                            )));
                            continue;
                        }
                    };

                    let mut rest = line[2..].iter();
                    let mut cont = String::new();
                    
                    while let Some(e) = rest.next(){
                        if let Token::Immediate(n) = e{
                            if line[2..].len() != 1{
                                if let Number::UInt8(n) = n{
                                    cont.push(*n as char);
                                }
                                else if let Number::Char(n) = n{
                                    cont.push(*n);
                                }
                                else {
                                    error = Some(RASMError::new(
                                        Some(line_count),
                                        ExType::Error,
                                        Some(Tokens(line.clone()).to_string()),
                                        Some(format!("Expected 8-bit number or characted, found: {}", n.to_string())),
                                        Some(format!("Try changing your number from 0 to 255 (uint) or a character"))
                                    ));
                                }
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
                            error = Some(RASMError::new(
                                Some(line_count),
                                ExType::Error,
                                Some(Tokens(line.clone()).to_string()),
                                Some(format!("Unknown token {:?} found in variable declaration", e)),
                                Some(format!("Try using string, character, uint8 instead of what you tried to use."))
                            ));
                        }
                    }
                    if let None = error{
                        node = Some(ASTNode::VarDec(VarDec{
                            name: var.clone().to_string(),
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
                        error = Some(RASMError::new(
                            Some(line_count),
                            ExType::Error,
                            Some(Tokens(line).to_string()),
                            Some(format!("Unexpected end of line after keyword !global, expected string, found nothing")),
                            Some(format!("consider adding something after global keyword"))
                        ));
                    }
                }
                Some(Token::Instruction(ins)) => {
                    if let Some((dst_raw, src_raw)) = vec_split_by(&line, Token::Comma){
                        let inst_dst = make_op(&dst_raw[1..]).ok();
                        let inst_src = make_op(&src_raw[1..]).ok();

                        node = Some(
                            ASTNode::Ins(ASTInstruction {ins: *ins, dst: inst_dst, src: inst_src, lin: line_count}));
                    }
                    else {
                        let operand = make_op(&line[1..]).ok();
                        node = Some(ASTNode::Ins(ASTInstruction {ins: *ins, dst: operand, src: None, lin: line_count}));
                    }
                },
                s => {
                    ast_tree.push(Err(RASMError::new(
                        Some(line_count),
                        ExType::Error,
                        Some(Tokens(line.clone()).to_string()),
                        Some(format!("Unexpected start of line: {:?}", s)),
                        Some(format!("consider starting line with Instruction, !global, section declaration or label declaration"))
                    )));
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

fn make_op(tok: &[Token]) -> Result<Operand, ()>{
    match tok.len(){
        1 => {
            Operand::try_from(tok[0].clone()).map_err(|_| ())
        },
        2 => {
            if let (Token::MemAddr(mem), Token::Keyword(kwd)) = (&tok[0], &tok[1]){
                match Mem::new(&mem, Some(*kwd)){
                    Ok(ma) => Ok(Operand::Mem(ma)),
                    Err(_) => Err(())
                }
            }
            else {
                Err(())
            }
        },
        _ => Err(())
    }
}
