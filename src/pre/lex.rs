// rasmx86_64 - src/core/lex.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use std::borrow::Cow;
use std::str::FromStr;
use crate::{
    pre::tok::Token,
    shr::{
        mem::Mem,
        ast::{
            ASTNode,
            Operand,
            Instruction,
        },
        var::{
            Variable,
            VarContent,
            VType,
        },
        symbol::Visibility,
        ins::Mnemonic as Mnm,
        num::Number,
        kwd::Keyword,
        error::RASMError,
        size::Size,
        segment::Segment
    }
};

pub struct Lexer;
impl Lexer{
    pub fn parse_file<'a>(file: Vec<Vec<Token>>) -> Vec<Result<(ASTNode<'a>, usize), RASMError>>{
        let mut line_count : i32 = -1;
        let mut ast_tree   : Vec<Result<(ASTNode, usize), RASMError>> = Vec::new();
        for line in file {
            line_count += 1;
            if line.is_empty(){
                continue;
            }
            
            let mut node  : Option<ASTNode> = None;
            let mut error : Option<RASMError>  = None;
            match line.get(0){
                Some(Token::Label(lbl)) => node = Some(ASTNode::Label(lbl.to_string())),
                Some(Token::Keyword(Keyword::Const|Keyword::Uninit|Keyword::Ronly)) => {
                    match make_var(Cow::Owned(line)){
                        Ok(var) => node = Some(ASTNode::Variable(var)),
                        Err(mut tmp_error) => {
                            tmp_error.set_line(line_count as usize);
                            error = Some(tmp_error)
                        }
                    }
                },
                Some(Token::Keyword(Keyword::Bits)) => {
                    if let Some(Token::Immediate(bits)) = line.get(1){
                        let uint8 = match bits.get_uint(){
                            Some(n) => match Number::squeeze_u64(n){
                                Number::UInt8(n) => n,
                                _ => {
                                    ast_tree.push(Err(RASMError::new(
                                        Some(line_count as usize),
                                        Some(format!("Couldn't fit number {} in 8-bits", n)),
                                        None
                                    )));
                                    continue
                                }
                            },
                            _ => {
                                ast_tree.push(Err(RASMError::new(
                                    Some(line_count as usize),
                                    Some(format!("Couldn't fit number {} in 8-bit unsigned integer", bits.to_string())),
                                    None
                                )));
                                continue
                            }
                        };
                        node = Some(ASTNode::Bits(uint8));
                    }
                    else {
                        error = Some(RASMError::new(
                            Some(line_count as usize),
                            Some(format!("Unexpected end of line after entry keyword, expected string, found nothing")),
                            Some(format!("Consider adding something after entry keyword"))
                        ));
                    }
                }
                Some(Token::Keyword(Keyword::Entry)) => {
                    if let Some(Token::String(entr)|Token::Unknown(entr)) = line.get(1){
                        node = Some(ASTNode::Entry(entr.to_string()));
                    }
                    else {
                        error = Some(RASMError::new(
                            Some(line_count as usize),
                            Some(format!("Unexpected end of line after entry keyword, expected string, found nothing")),
                            Some(format!("Consider adding something after entry keyword"))
                        ));
                    }
                }
                Some(Token::Keyword(Keyword::Extern)) => {
                    if let Some(Token::String(etrn)|Token::Unknown(etrn)) = line.get(1){
                        node = Some(ASTNode::Extern(etrn.to_string()));
                    }
                    else {
                        error = Some(RASMError::new(
                            Some(line_count as usize),
                            Some(format!("Unexpected end of line after extern keyword, expected string, found nothing")),
                            Some(format!("Consider adding something after extern keyword"))
                        ));
                    }
                }
                Some(Token::Keyword(Keyword::Global)) => {
                    if let Some(Token::String(glob)|Token::Unknown(glob)) = line.get(1){
                        node = Some(ASTNode::Global(glob.to_string()));
                    }
                    else {
                        error = Some(RASMError::new(
                            Some(line_count as usize),
                            Some(format!("Unexpected end of line after global keyword, expected string, found nothing")),
                            Some(format!("Consider adding something after global keyword"))
                        ));
                    }
                }
                Some(Token::Mnemonic(_)) => {
                    match make_ins(&line){
                        Ok(mut i) => {
                            i.line = line_count as usize;
                            node = Some(ASTNode::Ins(i));
                        }
                        Err(mut e) => {
                            e.set_line(line_count as usize);
                            error = Some(e);
                        }
                    }
                },
                t => {
                    println!("{:?}", t);
                    ast_tree.push(Err(RASMError::new(
                        Some(line_count as usize),
                        Some(format!("Unexpected start of line!")),
                        Some(format!("Consider starting line with Instruction, !global, section declaration or label declaration"))
                    )));
                }
            }

            if let Some(node_t) = node{
                ast_tree.push(Ok((node_t, line_count as usize)));
            }
            else if let Some(error_t) = error{
                ast_tree.push(Err(error_t));
            }

        }
        return ast_tree;
    }
}

fn make_ins(line: &[Token]) -> Result<Instruction, RASMError> {
    if line.len() == 0 {
        return Err(RASMError::new(
            None,
            Some(format!("Tried to make instruction from nothing")),
            None
        ))
    }
    let mut iter = line[0..].iter();
    let mut mnems : Vec<Mnm> = Vec::new();
    let mut tmp_buf : Vec<&Token> = Vec::new();
    
    while let Some(t) = iter.next(){
        if let Token::Mnemonic(m) = t{
            mnems.push(*m);
        }
        else {
            if t != &Token::Comma{
                tmp_buf.push(t);
            }
            break;
        }
    }

    let mut ops = Vec::new();
    while let Some(t) = iter.next(){
        if t == &Token::Comma{
            if tmp_buf.is_empty() == false{
                match make_op(&tmp_buf){
                    Ok(o) => {
                        ops.push(o);
                        tmp_buf = Vec::new();
                    },
                    Err(e) => return Err(e)
                }
            }
        }
        else {
            tmp_buf.push(t);
        }
    }
    if tmp_buf.is_empty() == false{
        match make_op(&tmp_buf){
            Ok(o) => {
                ops.push(o);
            },
            Err(e) => return Err(e)
        }
    }
    if mnems.len() == 0 {
        return Err(RASMError::new(
            None,
            Some(format!("Tried to make instruction with no mnemonics")),
            None
        ));
    }

    let addt = {
        match mnems.len() {
            1 => None,
            _ => Some(mnems[1..].to_vec())
        }
    };

    return Ok(Instruction{
        mnem: mnems[0],
        addt,
        oprs: ops,
        line: 0,
    });
}

fn make_op(line: &[&Token]) -> Result<Operand, RASMError>{
    if line.len() == 0 {
        return Err(RASMError::new(
            None,
            Some("Tried to make operand from nothing".to_string()),
            None
        ));
    }

    if line.len() == 1{
        match Operand::try_from(line[0]){
            Ok(o) => return Ok(o),
            Err(_) => return Err(RASMError::new(
                None,
                Some(format!("Failed to create operand from {}", line[0].to_string())),
                None
            ))
        }
    }

    if line.len() == 2 {
        match (&line[0], &line[1]){
             (Token::MemAddr(m), Token::Keyword(k))
            |(Token::Keyword(k), Token::MemAddr(m)) => {
                match Mem::new(&m, Some(*k)){
                    Ok(m) => return Ok(Operand::Mem(m)),
                    Err(e) => return Err(e),
                }
            },
             (Token::Segment(s), Token::Keyword(k))
            |(Token::Keyword(k), Token::Segment(s)) => {
                let size = match Size::try_from(*k){
                    Ok(s) => s,
                    Err(_) => return Err(RASMError::new(
                        None,
                        Some(format!("Couldn't parse size specifier `{}`", k.to_string())),
                        None
                    ))
                };
                let mem_new = match s.address{
                    Mem::Offset(b, o, _)        => Mem::Offset(b, o, size),
                    Mem::Direct(b, _)           => Mem::Direct(b, size),
                    Mem::Index (i, s, _)        => Mem::Index (i, s, size),
                    Mem::IndexOffset(i, s,o, _) => Mem::IndexOffset(i, s,o, size),
                    Mem::SIB   (b,i,s,_)        => Mem::SIB   (b, i, s, size),
                    Mem::RipRelative(o,_)       => Mem::RipRelative(o, size),
                    Mem::SIBOffset(b,i,s,o,_)   => Mem::SIBOffset(b,i,s,o,size),
                };
                return Ok(Operand::Segment(Segment{
                    segment: s.segment,
                    address: mem_new
                }));
            }
            _ => return Err(RASMError::new(
                None,
                Some(format!("Tried to make unexpected operand from two tokens; expected memory address along with size specifier")),
                None
            ))
        }
    }

    return Err(RASMError::new(
        None,
        Some(format!("Tried to make operand from too large set of tokens ({})", line.len())),
        None
    ))
}

fn make_var<'a>(line: Cow<'a, Vec<Token>>) -> Result<Variable<'a>, RASMError>{
    let vtype = match line.get(0) {
        Some(Token::Keyword(k)) => {
            match k {
                Keyword::Uninit => VType::Uninit,
                Keyword::Const  => VType::Const,
                Keyword::Ronly  => VType::Readonly,
                _               => return Err(RASMError::new(
                    None,
                    Some(format!("Unexpected keyword found at index 0; expected variable type")),
                    None
                ))
            }
        },
        Some(_) => return Err(RASMError::new(
            None,
            Some(format!("Unexpected token at index 0; expected variable type")),
            None
        )),
        None => return Err(RASMError::new(
            None,
            Some(format!("Expected variable type at index 0, found nothing")),
            Some(format!("Consider adding variable type on index like `!ronly` (.rodata), `!const` (.data) or `!uninit` (.bss)"))
        ))
    };

    let vname = match line.get(1){
        Some(t) => t.to_string(),
        None => return Err(RASMError::new(
            None,
            Some(format!("Expected variable name at index 1; found nothing")),
            None
        )),
    };

    let size = match line.get(2){
        Some(Token::Keyword(k)) => {
            match Size::try_from(*k){
                Ok(s) => <Size as Into<u8>>::into(s) as u32,
                Err(_) => return Err(RASMError::new(
                    None,
                    Some(format!("Couldn't parse keyword `{}` into size specifier", k.to_string())),
                    None
                )),
            }
        },
        Some(t) => {
            if vtype == VType::Uninit{
                if let Token::Immediate(n) = t{
                    match n.get_uint(){
                        Some(n) => n as u32,
                        None    => return Err(RASMError::new(
                            None,
                            Some(format!("Invalid size specifier at index 2; expected 32-bit unsigned integer")),
                            None
                        ))
                    }
                }
                else {
                    return Err(RASMError::new(
                        None,
                        Some(format!("Unexpected token found at index 2; expected keyword (!byte, !word, !dword or !qword) or 32-bit unsigned integer")),
                        None
                    ));
                }
            }
            else{
                return Err(RASMError::new(
                    None,
                    Some(format!("Unexpected token found at index 2; expected keyword (!byte, !word, !dword or !qword)")),
                    None
                ));
            }
        },
        None => return Err(RASMError::new(
            None,
            Some(format!("Expected variable name at index 2; found nothing")),
            None
        )),
    };

    let mut content = String::new();
    let mut iter = line[3..].iter();
    while let Some(i) = iter.next(){
        content.push_str(&i.to_string());
    }
    let content = par_str(content);
    match (&vtype, &content){
        (VType::Const|VType::Readonly, VarContent::String(_)|VarContent::Number(_))|
        (VType::Uninit, VarContent::Uninit) => {},
        (VType::Const|VType::Readonly, VarContent::Uninit) => 
        return Err(RASMError::new(
            None,
            Some("Variable type mismatch: declared variable is const/readonly, but content is undefined".to_string()),
            None
        )),
        (VType::Uninit, VarContent::String(_)|VarContent::Number(_)) => 
        return Err(RASMError::new(
            None,
            Some("Variable type mismatch: declared variable is uninitialized, but content is defined".to_string()),
            None,
        ))
    }
    return Ok(Variable{
        name: std::borrow::Cow::Owned(vname),
        vtype,
        size: size as u32,
        content,
        visibility: Visibility::Local,
    });
}

#[inline]
fn par_str<'a>(cont: String) -> VarContent<'a>{
    if let Ok(n) = Number::from_str(&cont){
        return VarContent::Number(n);
    }
    else if cont.is_empty(){
        return VarContent::Uninit;
    }
    else {
        return VarContent::String(Cow::Owned(cont.as_bytes().to_vec()));
    }
}
