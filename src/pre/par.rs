// rasmx86_64 - par.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

const EMPTY_STRING : &str = "";

use crate::{
    color::ColorText,
    pre::lex::LexErr,
    shr::ast::{
        AST,
        Label,
        ASTInstruction,
        Section,
        VarDec,
        ASTNode,
    }
};

pub struct Parser;

#[derive(Debug, Clone)]
pub enum ASTError{
    LexerError(LexErr),
    OutsideLabel(String),
    UnexpectedEnd
}

impl Parser{
    pub fn build_tree(list: Vec<Result<(ASTNode, usize), LexErr>>) -> Result<AST, Vec<ASTError>>{
        let mut errors : Vec<ASTError> = Vec::new();
        let mut ast = AST{sections: Vec::new(), text: Vec::new(), labels: Vec::new()};
        
        let mut inside_label : (bool, String) = (false, String::new());
        let mut vardecs      : Vec<VarDec>         = Vec::new();
        let mut instructions : Vec<ASTInstruction> = Vec::new();

        for node in list {
            match node {
                Err(error) => errors.push(ASTError::LexerError(error)),
                Ok(node) => {
                    match node.0 {
                        ASTNode::Label(lbl) => {
                            inside_label = (true, lbl)
                        },
                        ASTNode::VarDec(vdc) => {
                            vardecs.push(vdc);
                        }
                        ASTNode::Ins(ins) => {
                            if let (false, _) = inside_label{
                                errors.push(ASTError::OutsideLabel(format!(
                                            "at line {}: {} {:?} {:?}",
                                    node.1, 
                                    format!("{:?}", ins.ins).to_lowercase(),
                                    ins.dst, ins.src
                                )));
                            }
                            else {
                                instructions.push(ins);
                            }
                        },
                        ASTNode::Global(glob) => {
                            if let (false, EMPTY_STRING) = (inside_label.0, inside_label.clone().1.as_str()){
                                errors.push(ASTError::OutsideLabel(format!(
                                    "at line {}: !global {}", node.1, glob
                                )));
                            }
                            else if let (false, "text") = (inside_label.0, inside_label.clone().1.as_str()){
                                ast.text.push(glob)
                            }
                        },
                        ASTNode::Section(sec) => {
                            inside_label = (false, sec)
                        },
                        ASTNode::End => {
                            if let (false, EMPTY_STRING) = (inside_label.0, inside_label.clone().1.as_str()){
                                errors.push(ASTError::UnexpectedEnd);
                            }
                            else if let (true, label) = (inside_label.0, inside_label.clone().1){
                                ast.labels.push(Label{name: label, inst: instructions}); 
                                instructions = Vec::new();
                            }
                            else if let (false, "text") = (inside_label.0, inside_label.clone().1.as_str()){
                                continue
                            }
                            else if let (false, section) = (inside_label.0, inside_label.clone().1){
                                ast.sections.push(Section {name: section, vars: Some(vardecs)});
                                vardecs = Vec::new();
                            }
                        }
                    }
                }
            }
        }

        if errors.is_empty() == false{
            return Err(errors)
        }
        else{
            return Ok(ast);
        }
    }
}

impl ToString for ASTError{
    fn to_string(&self) -> String{
        match self{
            Self::LexerError(err) => err.to_string(),
            Self::OutsideLabel(string) => {
                format!("{}:\n\tSometing was found outside label/section!\n\t\t`{}`", "error".red(), string)
            },
            Self::UnexpectedEnd => {
                format!("{}:\n\t!end keyword was found outside label/section!", "error".red())
            }
        }
    }
}
