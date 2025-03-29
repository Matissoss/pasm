// rasmx86_64 - par.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    color::ColorText,
    pre::lex::{
        LexErr
    },
    shr::{
        mem::Mem,
        ins::Instruction,
        reg::Register,
        ast::{
            AST,
            Label,
            Operand,
            AstInstruction,
            Section,
            VarDec,
            AsmType,
            ASTNode,
            ExtASTNode
        }
    }
};

pub struct Parser;

#[derive(Debug)]
pub enum ASTError{
    LexerError(LexErr),
}

impl Parser{
    pub fn build_tree(list: Vec<Result<ASTNode, LexErr>>) -> Result<AST, Vec<ASTError>>{
        let mut errors : Vec<ASTError> = Vec::new();
        let mut ast = AST{sections: Vec::new(), global: Vec::new(), labels: Vec::new()};
        
        let mut inside_label : (bool, String) = (false, String::new());
        let mut vardecs      : Vec<VarDec>         = Vec::new();
        let mut instructions : Vec<AstInstruction> = Vec::new();
        for element in list{
            match element{
                Ok(node) => {
                    match node{
                        ASTNode::Label(name) => {
                            if let (false, sec) = inside_label{
                                ast.sections.push(Section{name:sec, vars: Some(vardecs)});
                                vardecs = Vec::new();
                                inside_label = (true, name);
                            }
                            else if let (true, _) = inside_label{
                                ast.labels.push(Label{name: inside_label.1, inst: instructions});
                                instructions = Vec::new();
                                inside_label = (true, name);
                            }
                        },
                        ASTNode::Ins(ins) => instructions.push(ins),
                        ASTNode::Section(str) => {
                            if let (false, prev) = inside_label{
                                if !prev.is_empty() {
                                    ast.sections.push(Section{name: prev, vars: Some(vardecs)});
                                    vardecs = Vec::new();
                                }
                            }
                            inside_label = (false, str);
                        },
                        ASTNode::Global(glb) => ast.global.push(glb),
                        ASTNode::VarDec(vdc) => vardecs.push(vdc),
                    }
                },
                Err(error) => errors.push(ASTError::LexerError(error))
            }
        }
        
        if let (true, str) = inside_label{
            ast.labels.push(Label{name: str, inst: instructions});
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
            Self::LexerError(err) => err.to_string()
        }
    }
}
