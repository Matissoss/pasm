// rasmx86_64 - par.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

const EMPTY_STRING : &str = "";

use crate::{
    shr::ast::{
        AST,
        Instruction,
        VarDec,
        ASTNode,
        Label
    },
    shr::error::{
        RASMError,
        ExceptionType as ExType
    }
};

pub struct Parser;

type LexTree= Vec<Result<(ASTNode, usize), RASMError>>;
impl Parser{
    pub fn build_tree(list: LexTree) -> Result<AST, Vec<RASMError>>{
        let mut errors : Vec<RASMError> = Vec::new();
        let mut ast = AST{vars: Vec::new(), global: Vec::new(), labels: Vec::new(), bits: 64, entry: "_start".to_string()};
        
        let mut inside_label : (bool, String) = (false, String::new());
        let mut vardecs      : Vec<VarDec>         = Vec::new();
        let mut instructions : Vec<Instruction> = Vec::new();

        for node in list {
            match node {
                Err(error) => errors.push(error),
                Ok(node) => {
                    match node.0 {
                        ASTNode::Entry(ref ent) => {
                            if ast.entry == "_start".to_string(){
                                ast.entry = ent.to_string();
                            }
                            else {
                                errors.push(RASMError::new(
                                    Some(node.1),
                                    ExType::Error,
                                    Some(format!("{:?}", node)),
                                    Some(format!("Entry was already defined!")),
                                    Some(format!("Consider removing entry."))
                                ))
                            }
                        }
                        ASTNode::Label(lbl) => {
                            if !instructions.is_empty(){
                                ast.labels.push(Label {
                                    name: inside_label.1,
                                    inst: instructions,
                                });
                                instructions = Vec::new();
                            }
                            inside_label = (true, lbl)
                        },
                        ASTNode::VarDec(vdc) => {
                            vardecs.push(vdc);
                        }
                        ASTNode::Ins(ins) => {
                            if !inside_label.0{
                                errors.push(RASMError::new(
                                    Some(node.1),
                                    ExType::Error,
                                    Some(format!("{:?} {:?} {:?}", ins.mnem, ins.dst(), ins.src())),
                                    Some(format!("This instruction was outside of label!")),
                                    Some(format!("RASM doesn't support instructions outside of label. Consider adding it to label like: _misc or something like this"))
                                ));
                            }
                            else {
                                instructions.push(ins);
                            }
                        },
                        ASTNode::Global(glob) => {
                            if let (false, EMPTY_STRING) = (inside_label.0, inside_label.clone().1.as_str()){
                                errors.push(RASMError::new(
                                    Some(node.1),
                                    ExType::Error,
                                    Some(format!("{}", glob)),
                                    Some(format!("This global statement was outside of section!")),
                                    Some(format!("RASM doesn't support globals outside of section. Consider adding it to section .text"))
                                ));
                            }
                            else if let (false, "text") = (inside_label.0, inside_label.clone().1.as_str()){
                                ast.global.push(glob)
                            }
                        },
                        ASTNode::Section(sec) => {
                            inside_label = (false, sec)
                        },
                    }
                }
            }
        }

        if !instructions.is_empty(){
            ast.labels.push(Label {
                name: inside_label.1,
                inst: instructions
            });
        }
        ast.vars = vardecs;
        if errors.is_empty() == false{
            return Err(errors)
        }
        else{
            return Ok(ast);
        }
    }
}
