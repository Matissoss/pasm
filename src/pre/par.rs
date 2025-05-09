// rasmx86_64 - src/pre/par.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use std::borrow::Cow;

const EMPTY_STRING: &str = "";
use crate::shr::{
    ast::{ASTNode, Instruction, Label, AST},
    error::RASMError,
    symbol::Visibility,
    var::Variable,
};

pub struct Parser;

type LexTree<'a> = Vec<Result<(ASTNode<'a>, usize), RASMError>>;
impl Parser {
    pub fn build_tree(list: LexTree) -> Result<AST, Vec<RASMError>> {
        let mut errors: Vec<RASMError> = Vec::new();
        let mut ast = AST {
            bits: None,
            entry: None,
            vars: Vec::new(),
            labels: Vec::new(),
            globals: Vec::new(),
            externs: Vec::new(),
        };

        let mut inside_label: (bool, String) = (false, String::new());
        let mut vardecs: Vec<Variable> = Vec::new();
        let mut instructions: Vec<Instruction> = Vec::new();

        for node in list {
            match node {
                Err(error) => errors.push(error),
                Ok(node) => {
                    match node.0 {
                        ASTNode::Label(lbl) => {
                            if !instructions.is_empty() {
                                ast.labels.push(Label {
                                    name: Cow::Owned(inside_label.1),
                                    inst: instructions,
                                    visibility: Visibility::Local,
                                });
                                instructions = Vec::new();
                            }
                            inside_label = (true, lbl)
                        }
                        ASTNode::Variable(var) => vardecs.push(var),
                        ASTNode::Ins(ins) => {
                            if !inside_label.0 {
                                errors.push(RASMError::with_tip(
                                    Some(node.1),
                                    Some("This instruction was outside of label!"),
                                    Some("RASM doesn't support instructions outside of label. Consider adding it to label like: _misc or something like this")
                                ));
                            } else {
                                instructions.push(ins);
                            }
                        }
                        ASTNode::Global(glob) => {
                            if (inside_label.0, inside_label.1.as_str()) == (false, EMPTY_STRING) {
                                ast.globals.push(glob);
                            } else {
                                errors.push(RASMError::no_tip(
                                    Some(node.1),
                                    Some("Globals can be only declared outside of labels, not inside of"),
                                ));
                            }
                        }
                        ASTNode::Extern(extrn) => {
                            if (inside_label.0, inside_label.1.as_str()) == (false, EMPTY_STRING) {
                                ast.externs.push(extrn);
                            } else {
                                errors.push(RASMError::no_tip(
                                    Some(node.1),
                                    Some("Externs can be only declared outside of labels, not inside of"),
                                ));
                            }
                        }
                        ASTNode::Entry(entry) => {
                            if (inside_label.0, inside_label.1.as_str()) == (false, EMPTY_STRING) {
                                if ast.entry.is_none() {
                                    ast.entry = Some(entry);
                                } else {
                                    errors.push(RASMError::no_tip(
                                        Some(node.1),
                                        Some("Entry point declared twice"),
                                    ));
                                }
                            } else {
                                errors.push(RASMError::no_tip(
                                    Some(node.1),
                                    Some("Entries can be only declared outside of labels, not inside of"),
                                ));
                            }
                        }
                        ASTNode::Bits(bits) => {
                            if (inside_label.0, inside_label.1.as_str()) == (false, EMPTY_STRING) {
                                if ast.bits.is_none() {
                                    match bits{
                                        16|32|64 => ast.bits = Some(bits),
                                        n        =>
                                            errors.push(RASMError::no_tip(
                                                Some(node.1),
                                                Some(format!("Invalid bits specifier; expected 16, 32, 64, found {}", n)),
                                            ))
                                    }
                                } else {
                                    errors.push(RASMError::no_tip(
                                        Some(node.1),
                                        Some("Program bits declared twice!"),
                                    ));
                                }
                            } else {
                                errors.push(RASMError::no_tip(
                                    Some(node.1),
                                    Some("Bits can be only declared outside of labels, not inside of"),
                                ));
                            }
                        }
                    }
                }
            }
        }

        if !instructions.is_empty() {
            ast.labels.push(Label {
                name: Cow::Owned(inside_label.1),
                inst: instructions,
                visibility: Visibility::Local,
            });
        }
        ast.vars = vardecs;
        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(ast)
        }
    }
}
