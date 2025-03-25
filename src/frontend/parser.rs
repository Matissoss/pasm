//  rasmx86_64  -   parser.rs
//  -------------------------
//  made by matissoss
//  licensed under MPL 2.0

use std::{
    str::FromStr,
    //path::PathBuf
};

use crate::{
    cli::CLI,
    frontend::{
        label   ::Label,
        register::{
            Register,
            RegErr
        }
    }
};

#[allow(unused)]
pub struct SourceCode{
    labels: Vec<Label>
}

#[allow(unused)]
#[derive(Debug)]
pub enum Operand{
    Immediate(i64),
    Register(Register),
    MemAddr {
        base_reg     : Register,
        offset       : i64
    },
    None
}

#[allow(unused)]
#[derive(Debug)]
pub enum LOC{
    Label(String),
    Jump(String, String),
    Instruction {
        ins: String, // for now String, later replaced by enum
        des: Option<Operand>,
        src: Option<Operand>,
    },
    Blank
}

pub struct Parser;

impl Parser{
    fn split_line(line: &str) -> Result<[Option<String>; 3], &str> {
        let mut inside_closure : (bool, char)           = (false, ' ');
        let mut to_return      : [Option<String>; 3]    = [None, None, None];
        let mut current_index  : usize                  = 0;
        let mut tmp_buf        : Vec<char>              = Vec::new();
        let bytes = line.trim().as_bytes();
        for b in bytes{
            match (inside_closure, *b as char){
                (_, ';') => break,
                (_, ',') => {
                    if current_index < to_return.len() {
                        to_return[current_index] = Some(String::from_iter(tmp_buf.iter()));
                        current_index += 1;
                    }
                    else{
                        return Err("[src/frontend/parser.rs:split_line] (ERROR): current_index < to_return.len() is false!");
                    }
                    tmp_buf = Vec::new();
                    inside_closure = (true, ',')
                }
                ((true, ','), ' '|'\t') => continue,
                ((false, ' '), '[') => {
                    tmp_buf.push('[');
                    inside_closure = (true, '[');
                },
                ((true , '['), ' '|'\t') => continue,
                ((true , '['), c)  => tmp_buf.push(c),
                (_, ' ') => {
                    if current_index < to_return.len() {
                        to_return[current_index] = Some(String::from_iter(tmp_buf.iter()));
                        current_index += 1;
                    }
                    else{
                        return Err("[src/frontend/parser.rs:split_line] (ERROR): current_index < to_return.len() is false!");
                    }
                    tmp_buf = Vec::new();
                }
                (_, c) => tmp_buf.push(c)
            }
        }
        if tmp_buf.is_empty() == false && current_index < to_return.len(){
            to_return[current_index] = Some(String::from_iter(tmp_buf.iter()));
        }
        return Ok(to_return);
    }
    pub fn parse_line(line: &str) -> Option<LOC>{
        match Self::split_line(line){
            Ok([ins, des, src]) => {
                if let (Some(des), Some(src), Some(ins)) = (&des, &src, &ins){
                    match (Operand::from_str(&des), Operand::from_str(&src)){
                        (Ok(des_op), Ok(src_op)) => {
                            return Some(LOC::Instruction{
                                ins: ins.to_string(),
                                des: Some(des_op),
                                src: Some(src_op)
                            });
                        }
                        (Err(des_err), Err(src_err)) => {
                            CLI.error("src/frontend/parser.rs", "parse_line.rs", &des_err.to_string());
                            CLI.error("src/frontend/parser.rs", "parse_line.rs", &src_err.to_string());
                        }
                        (_, Err(src_err)) => {
                            CLI.error("src/frontend/parser.rs", "parse_line.rs", &src_err.to_string());
                        }
                        (Err(des_err), _) => {
                            CLI.error("src/frontend/parser.rs", "parse_line.rs", &des_err.to_string());
                        }
                    }
                }
                if let Some(ins) = ins {
                    if let Some(dest) = des{
                        if let Ok(dest) = Operand::from_str(&dest){
                            return Some(LOC::Instruction{
                                ins,
                                des: Some(dest),
                                src: None
                            });
                        }
                        else if ins.starts_with('j'){
                            return Some(LOC::Jump(ins,dest.to_string()))
                        }
                    }
                    if ins.ends_with(':'){
                        return Some(LOC::Label(ins))
                    }
                    return Some(LOC::Instruction {
                        ins,
                        des: None,
                        src: None
                    });
                }
            },
            Err(error) => {
                CLI.error("src/frontend/parser.rs", "parse_line.rs", error);
            }
        }
        return None;
    }
}


//  trait impl's

pub enum OprErr{
    RegisterErr(RegErr),
    UnexpectedChar(char),
    NAN(String),
    OffsetNotFound,
    RegisterNotFound,
    AString,
    Blank
}

impl ToString for OprErr{
    fn to_string(&self) -> String {
        match &self{
            Self::RegisterErr(err) => err.to_string(),
            Self::UnexpectedChar(c)=> format!("E-SYN-0005: Unexpected character found inside operand: `{}`;", c),
            Self::NAN(str)         => format!("E-SYN-0006: `{}` is not a number;", str),
            Self::OffsetNotFound   => format!("E-SYN-0007: Offset not found in memory address;"),
            Self::RegisterNotFound => format!("E-SYN-0008: Register wasn't found in memory address;"),
            Self::AString          => format!("E-SYN-0009: String was found inside operand, which is not allowed;"),
            Self::Blank            => format!("E-SYN-0010: Operand is blank :)")
        }
    }
}

impl FromStr for Operand{
    type Err = OprErr;
    fn from_str(str: &str) -> Result<Self, <Self as FromStr>::Err> {
        let bytes = str.as_bytes();
        
        if let Ok(reg) = Register::from_str(str){
            return Ok(Self::Register(reg));
        }

        if bytes.len() >= 1{
            if bytes[0] == '[' as u8{
                let mut tmp_buf = Vec::new();
                let mut tr_reg : Option<Register> = None;
                for b in bytes{
                    let c = *b as char;
                    match c{
                        '[' => continue,
                        ']' => break,
                        '-'|'+' => {
                            match Register::from_str(&String::from_iter(tmp_buf.iter())){
                                Ok(reg) => {
                                    tr_reg = Some(reg);
                                    tmp_buf = Vec::new();
                                    tmp_buf.push(c);
                                },
                                Err(err) => return Err(OprErr::RegisterErr(err))
                            }
                        },
                        ' ' => {
                            return Err(OprErr::UnexpectedChar(' '))
                        }
                        _ => tmp_buf.push(c)
                    }
                }
                let tr_offset = if tmp_buf.is_empty() == false{
                    if let Ok(numb) = String::from_iter(tmp_buf.iter()).parse::<i64>(){
                        numb
                    }
                    else {
                        return Err(OprErr::NAN(String::from_iter(tmp_buf.iter())));
                    }
                }
                else {return Err(OprErr::OffsetNotFound)};
                if let Some(reg) = tr_reg{
                    return Ok(Self::MemAddr {
                        base_reg: reg,
                        offset  : tr_offset
                    });
                }
                else {
                    return Err(OprErr::RegisterNotFound);
                }
            }
            else {
                if bytes[0] == '\'' as u8{
                    return Ok(Self::Immediate(bytes[1] as i64))
                }
                else if bytes[0] == '"' as u8{
                    return Err(OprErr::AString);
                }
                else {
                    return if let Ok(numb) = str.parse::<i64>(){
                        return Ok(Self::Immediate(numb))
                    }
                    else {
                        Err(OprErr::NAN(str.to_string()))
                    }
                }
            }
        }
        return Err(OprErr::Blank);
    } 
}


// tests
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn split_line_test(){
        let line = Parser::split_line("     mov [ rax - 20 ], 100 ; commentarz");
        assert!(Ok([Some("mov".to_string()), Some("[rax-20]".to_string()), Some("100".to_string())]) == line);
        let line1 = Parser::split_line("     mov [ rax - 20 ], '1' ; commentarz");
        assert!(Ok([Some("mov".to_string()), Some("[rax-20]".to_string()), Some("'1'".to_string())]) == line1);
    }
}
