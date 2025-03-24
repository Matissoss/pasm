//  rasmx86_64  -   parser.rs
//  -------------------------
//  made by matissoss
//  licensed under MPL 2.0

use std::{
    str::FromStr,
    path::PathBuf
};

use crate::global::Orb;
use crate::frontend::{
    label   ::Label,
    register::*
};

#[derive(Default)]
pub struct SourceCode{
    labels: Vec<Label>
}

// loc - line of code
#[derive(Debug)]
pub struct LOC{
    // for now &str, next struct Instruction
    instruction : String,
    source      : Option<Orb<Register, String>>,
    destination : Option<Orb<Register, Label >>
}

pub struct Parser;

impl Parser{
    pub fn from_file(_file: &PathBuf) -> SourceCode{
        return SourceCode::default();
    }

    fn debloat_line(line: &str) -> Vec<String>{
        let mut to_return = Vec::new();
        let mut inside_closure : (bool, char) = (false, ' ');
        let mut tmp_buf : Vec<char> = Vec::new();
        for byte in line.as_bytes(){
            match (inside_closure, *byte as char){
                ((true, '['), ']') => {
                    tmp_buf.push(']');
                    to_return.push(String::from_iter(tmp_buf.iter()));
                    tmp_buf = Vec::new();
                    inside_closure = (false, ' ');
                },
                ((true, '\''), '\'') => {
                    tmp_buf.push('\'');
                    to_return.push(String::from_iter(tmp_buf.iter()));
                    tmp_buf = Vec::new();
                    inside_closure = (false, ' ');
                }
                ((true, '"'), '"') => {
                    tmp_buf.push('"');
                    to_return.push(String::from_iter(tmp_buf.iter()));
                    tmp_buf = Vec::new();
                    inside_closure = (false, ' ');
                }
                ((false,' '), '\t') | ((false, ' '), ' ') => {
                    if tmp_buf.is_empty() == false{
                        to_return.push(String::from_iter(tmp_buf.iter()));
                        tmp_buf = Vec::new();
                    }
                    continue;
                }
                ((false, ' '), c) => {
                    match c {
                        '['|'\''|'"' => {
                            tmp_buf.push(c);
                            inside_closure = (true, c);
                            continue;
                        }
                        '\n'|',' => {
                            if tmp_buf.is_empty() == false {
                                to_return.push(String::from_iter(tmp_buf.iter()));
                                tmp_buf = Vec::new();
                            }
                            continue;
                        },
                        ';' => {
                            if tmp_buf.is_empty() == false {
                                to_return.push(String::from_iter(tmp_buf.iter()));
                            }
                            return to_return;
                        },
                        _ => tmp_buf.push(c)
                    }
                }
                _ => continue
            }
        }
        if tmp_buf.is_empty() == false {
            to_return.push(String::from_iter(tmp_buf.iter()));
        }
        return to_return;
    }
    pub fn parse_line(line: &str) -> Result<Orb<LOC, Label>, ()>{
        let debloated_line = Self::debloat_line(line);
        if debloated_line.len() == 3 {
            let src = if let Ok(reg) = Register::from_str(&debloated_line[2]){
                Orb::A(reg)
            }
            else{
                Orb::B(debloated_line[2].clone())
            };
            let dest = if let Ok(reg) = Register::from_str(&debloated_line[1]){
                reg
            }
            else {return Err(())};
            return Ok(Orb::A(LOC {
                instruction : debloated_line[0].clone(),
                source      : Some(src),
                destination : Some(Orb::A(dest)),
            }));
        }
        else if debloated_line.len() == 1 && debloated_line[0].ends_with(':') {
            return Ok(Orb::B(Label {
                name : debloated_line[0].clone(),
                instructions: Vec::new()
            }))
        }
        else if debloated_line.len() == 2 {
            if debloated_line[0].starts_with('j'){
                return Ok(Orb::A(LOC{
                    instruction: debloated_line[0].clone(),
                    source: None,
                    destination: Some(Orb::B(Label {name: debloated_line[1].clone(), instructions: Vec::new()}))
                }));
            }
            else if debloated_line[0] == "call" {
                return Ok(Orb::A(LOC{
                    instruction: debloated_line[0].clone(),
                    source: None,
                    destination: Some(Orb::B(Label {name: debloated_line[1].clone(), instructions: Vec::new()}))
                }));
            }
        }
        else if debloated_line.len() == 1 {
            return Ok(Orb::A(LOC {
                instruction: debloated_line[0].clone(),
                source: None,
                destination: None
            }))
        }
        Err(())
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn parse_line_test(){
        let line : LOC = match Parser::parse_line("  mov rax, 10 ; comment").unwrap(){
            Orb::A(loc) => loc,
            _ => panic!("[NOT A LOC]!!!")
        };
        println!("LINE: {:?}", line);
        assert!(line.instruction      == "mov");
        assert!(line.source           == Some( Orb::B( "10".to_string() ) ) );
        assert!(line.destination      == Some( Orb::A(Register::Bit64(Register64bit::RAX))) );
        drop(line);
        let line1 : LOC = match Parser::parse_line("  mov rax,14 ; comment 1").unwrap(){
            Orb::A(loc) => loc,
            _ => panic!("[NOT A LOC]!!!")
        };
        assert!(line1.instruction      == "mov");
        assert!(line1.source           == Some( Orb::B( "14".to_string() ) ) );
        assert!(line1.destination      ==  Some( Orb::A(Register::Bit64(Register64bit::RAX))) );
        drop(line1);
        let label : Label = match Parser::parse_line("_start:").unwrap(){
            Orb::A(_) => panic!("THIS IS A LABEL!!!"),
            Orb::B(label) => label
        }; 
        assert!(label.name == "_start:".to_string());
        assert!(label.instructions == Vec::new());
    }
}
