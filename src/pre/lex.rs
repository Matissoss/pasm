// rasmx86_64 - lex.rs
// -------------------
// made by matissoss
// licensed under MPL

use crate::{
    pre::tok::Token,
    conf::PREFIX_VAL,
    shr::{
        mem::Mem,
        ast::{
            ASTNode,
            Operand,
            Instruction,
            VarDec,
        },
        ins::Mnemonic as Mnm,
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
                Some(Token::Section(sec))           => node = Some(ASTNode::Section(sec.to_string())),
                Some(Token::Keyword(Keyword::Entry)) => {
                    match line.get(1) {
                        Some(Token::SymbolRef(r)) => {
                            node = Some(ASTNode::Entry(r.to_string()));
                        },
                        None => error = Some(RASMError::new(
                            Some(line_count),
                            ExType::Error,
                            Some(format!("Expected labelref after !entry keyword, found nothing")),
                            Some(format!("Consider adding labelref like: `!entry @_start`"))
                        )),
                        Some(t) => error = Some(RASMError::new(
                            Some(line_count),
                            ExType::Error,
                            Some(format!("Expected labelref after !entry keyword, found {}", t.to_string())),
                            Some(format!("Consider changing labelref like: `!entry @_start`"))
                        ))
                    }
                }
                Some(Token::Keyword(Keyword::Const|Keyword::Uninit)) => {
                    match make_var(&line){
                        Ok(var) => node = Some(ASTNode::VarDec(var)),
                        Err(mut tmp_error) => {
                            tmp_error.set_line(line_count);
                            error = Some(tmp_error)
                        }
                    }
                }
                Some(Token::Keyword(Keyword::Global)) => {
                    if let Some(Token::String(glob)|Token::Unknown(glob)) = line.get(1){
                        node = Some(ASTNode::Global(glob.to_string()));
                    }
                    else {
                        error = Some(RASMError::new(
                            Some(line_count),
                            ExType::Error,
                            Some(format!("Unexpected end of line after keyword !global, expected string, found nothing")),
                            Some(format!("consider adding something after global keyword"))
                        ));
                    }
                }
                Some(Token::Mnemonic(mnm)) => {
                    let mut mnems : Vec<Mnm> = Vec::new();
                    mnems.push(*mnm);

                    let mut iter = line[1..].iter();
                    let mut index = 1;
                    while let Some(Token::Mnemonic(tmp_mnm)) = iter.next(){
                        mnems.push(*tmp_mnm);
                        index += 1;
                    }
                    let mut operands : Vec<Operand> = Vec::new();
                    for raw_operand in split_vec(line[index..].to_vec(), Token::Comma){
                        match make_op(&raw_operand){
                            Ok(operand) => operands.push(operand),
                            Err(mut err)  => {
                                err.set_line(line_count);
                                ast_tree.push(Err(err));
                                break;
                            }
                        }
                    }
                    let addt : Option<Vec<Mnm>> = if mnems.len() == 1 {
                        None
                    }
                    else{
                        Some(mnems[1..].to_vec())
                    };
                    node = Some(ASTNode::Ins(Instruction{
                        mnem: mnems[0],
                        oprs: operands,
                        line: line_count,
                        addt,
                    }))
                },
                s => {
                    ast_tree.push(Err(RASMError::new(
                        Some(line_count),
                        ExType::Error,
                        Some(format!("Unexpected start of line: {:?}", s)),
                        Some(format!("Consider starting line with Instruction, !global, section declaration or label declaration"))
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

fn split_vec(tok_vec: Vec<Token>, by: Token) -> Vec<Vec<Token>>{
    let mut tokens = Vec::new();
    
    let mut tmp_buf = Vec::new();
    for tok in tok_vec{
        if tok == by {
            tokens.push(tmp_buf);
            tmp_buf = Vec::new();
        }
        else {
            tmp_buf.push(tok);
        }
    }

    if !tmp_buf.is_empty(){
        tokens.push(tmp_buf);
    }

    return tokens;
}

fn make_op(tok: &[Token]) -> Result<Operand, RASMError>{
    match tok.len(){
        0 => return Err(RASMError::new(
            None,
            ExType::Error,
            Some(format!("Tried to make operand from '' (blank)")),
            Some(format!("Consider adding operand to one of sides (destination or source) of instruction."))
        )),
        1 => {
            match Operand::try_from(tok[0].clone()){
                Ok(op) => Ok(op),
                Err(_) => {
                    return Err(RASMError::new(
                        None,
                        ExType::Error,
                        Some(format!("Couldn't parse following tokens into operand")),
                        Some(format!("Try formatting as constref, labelref, immediate, register or memory\n\t     (or maybe you did forgot to add size specifier after memory?)"))
                    ));
                }
            }
        },
        2 => {
            if let (Token::MemAddr(mem), Token::Keyword(kwd)) = (&tok[0], &tok[1]){
                match Mem::new(&mem, Some(*kwd)){
                    Ok(ma) => Ok(Operand::Mem(ma)),
                    Err(err) => Err(err)
                }
            }
            else {
                Err(RASMError::new(
                    None,
                    ExType::Error,
                    Some(format!("Unexpected tokens found")),
                    Some(format!("Expected memory address and size specifier"))
                ))
            }
        },
        _ => {
            println!("{:?}", tok);
            Err(RASMError::new(
                None,
                ExType::Error,
                Some(format!("Too much tokens were found")),
                Some(format!("Expected (at most) 2 tokens, found more."))
            ))
        }
    }
}

fn make_var(line: &[Token]) -> Result<VarDec, RASMError>{
    match line.get(0){
        Some(t) => {
            match t {
                Token::Keyword(Keyword::Uninit) => {
                    match line.get(1) {
                        Some(Token::Unknown(var)) =>{
                            let mut size : usize = 0;
                            if let Some(tmp_size) = line.get(2){
                                match tmp_size {
                                    Token::Immediate(n) => {
                                        if let Some(i) = n.get_uint(){
                                            if let Ok(n) = i.try_into() {
                                                size = n;
                                            }
                                            else {
                                                return Err(RASMError::new(
                                                    None,
                                                    ExType::Error,
                                                    Some(format!("Size specifier doesn't fit into `usize` Rust type")),
                                                    Some(format!(
                                                        "Consider changing size specifier into number from {} to {}",
                                                        usize::MIN, usize::MAX
                                                    ))
                                                ))
                                            }
                                        }
                                        else {
                                            return Err(RASMError::new(
                                                None,
                                                ExType::Error,
                                                Some(format!(
                                                    "Expected size specifier to be of type uint, found: {}", n.to_string())),
                                                Some(format!(
                                                    "Consider changing size specifier into uint like `{}10`", PREFIX_VAL))
                                            ))
                                        }
                                    },
                                    y => return Err(RASMError::new(
                                        None,
                                        ExType::Error,
                                        Some(format!(
                                            "Unexpected token at index 2; found `{}`, expected immediate",
                                            y.to_string()
                                        )),
                                        Some(format!(
                                            "Consider changing token `{}` to immediate like: `{}10`",
                                            y.to_string(), PREFIX_VAL
                                        ))
                                    ))
                                }
                            }

                            return Ok(VarDec {
                                name: var.to_string(),
                                size,
                                bss: true,
                                content: None,
                            })
                        },
                        None => Err(RASMError::new(
                            None,
                            ExType::Error,
                            Some(format!("Expected string at token of index 1, found nothing.")),
                            None
                        )),
                        Some(y) => Err(RASMError::new(
                            None,
                            ExType::Error,
                            Some(format!("Expected string at token of index 1, found `{}`.", y.to_string())),
                            None
                        )),
                    }
                },
                Token::Keyword(Keyword::Const) => {
                    match line.get(1) {
                        Some(Token::Unknown(var)) =>{
                            let size : usize = match line.get(2) {
                                Some(Token::Keyword(Keyword::Qword)) => 8,
                                Some(Token::Keyword(Keyword::Dword)) => 4,
                                Some(Token::Keyword(Keyword::Word))  => 2,
                                Some(Token::Keyword(Keyword::Byte))  => 1,
                                _   => {
                                    return Err(RASMError::new(
                                    None,
                                    ExType::Error,
                                    Some(format!("Found wrong size specifier for variable")),
                                    Some(format!(
                                        "Place either one of these after variable declaration: !byte, !word, !dword, !qword"))
                                    ));
                                }
                            };
                            let mut rest = line[3..].iter();
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
                                            return Err(RASMError::new(
                                                None,
                                                ExType::Error,
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
                                    return Err(RASMError::new(
                                        None,
                                        ExType::Error,
                                        Some(format!("Unknown token {:?} found in variable declaration", e)),
                                        Some(format!("Try using string, character, uint8 instead of what you tried to use."))
                                    ));
                                }
                            }
                            return Ok(VarDec{
                                name: var.to_string(),
                                bss: false,
                                size,
                                content: Some(cont)
                            })
                        },
                        None => Err(RASMError::new(
                            None,
                            ExType::Error,
                            Some(format!("Expected string at token of index 1, found nothing.")),
                            None
                        )),
                        Some(y) => Err(RASMError::new(
                            None,
                            ExType::Error,
                            Some(format!("Expected string at token of index 1, found `{}`.", y.to_string())),
                            None
                        )),
                    }
                },
                _ => Err(RASMError::new(
                    None,
                    ExType::Error,
                    Some(format!("Unexpected start of line: expected !data or !bss keyword.")),
                    None
                ))
            }
        }
        None => Err(RASMError::new(
            None,
            ExType::Error,
            Some(format!("Unexpected EOL")),
            None
        ))
    }
}
