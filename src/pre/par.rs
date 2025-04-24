// rasmx86_64 - par.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

const EMPTY_STRING : &str = "";

use crate::shr::{
    ast::{
        AST,
        Instruction,
        ASTNode,
        Label
    },
    var::Variable,
    error::RASMError,
    symbol::Visibility
};

pub struct Parser;

type LexTree= Vec<Result<(ASTNode, usize), RASMError>>;
impl Parser{
    pub fn build_tree(list: LexTree) -> Result<AST, Vec<RASMError>>{
        let mut errors : Vec<RASMError> = Vec::new();
        let mut ast = AST{vars: Vec::new(), global: Vec::new(), labels: Vec::new()};
        
        let mut inside_label : (bool, String)   = (false, String::new());
        let mut vardecs      : Vec<Variable>    = Vec::new();
        let mut instructions : Vec<Instruction> = Vec::new();

        for node in list {
            match node {
                Err(error) => errors.push(error),
                Ok(node) => {
                    match node.0 {
                        ASTNode::Label(lbl) => {
                            if !instructions.is_empty(){
                                ast.labels.push(Label {
                                    name: inside_label.1,
                                    inst: instructions,
                                    visibility: Visibility::Local,
                                });
                                instructions = Vec::new();
                            }
                            inside_label = (true, lbl)
                        },
                        ASTNode::Entry(_) => {}
                        ASTNode::Variable(var) => vardecs.push(var),
                        ASTNode::Ins(ins) => {
                            if !inside_label.0{
                                errors.push(RASMError::new(
                                    Some(node.1),
                                    Some(format!("This instruction was outside of label!")),
                                    Some(format!("RASM doesn't support instructions outside of label. Consider adding it to label like: _misc or something like this"))
                                ));
                            }
                            else {
                                instructions.push(ins);
                            }
                        },
                        ASTNode::Global(glob) => {
                            if (inside_label.0, inside_label.1.as_str()) == (false, EMPTY_STRING){
                                ast.global.push(glob);
                            }
                            else {
                                errors.push(RASMError::new(
                                    Some(node.1),
                                    Some(format!("Globals can be only declared outside of labels, not inside of")),
                                    None
                                ));
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
                inst: instructions,
                visibility: Visibility::Local,
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
