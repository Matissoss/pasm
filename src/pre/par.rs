// rasmx86_64 - src/pre/par.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

const EMPTY_STRING: &str = "";
use crate::shr::{
    ast::{ASTNode, Instruction, Label, AST},
    error::RASMError,
    symbol::Visibility,
};

pub struct Parser;

type LexTree = Vec<Result<(ASTNode, usize), RASMError>>;
impl Parser {
    pub fn build_tree(list: LexTree) -> Result<AST, Vec<RASMError>> {
        let mut errors: Vec<RASMError> = Vec::new();
        let mut ast = AST::default();
        let mut tmp_attributes: Vec<String> = Vec::new();
        let mut inside_label: (bool, String) = (false, String::new());
        let mut instructions: Vec<Instruction> = Vec::new();

        for node in list {
            match node {
                Err(error) => errors.push(error),
                Ok(node) => {
                    match node.0 {
                        ASTNode::Attributes(s) => {
                            // we need to assert that attribute means end of label
                            if !instructions.is_empty() {
                                let (bits, align, global) =
                                    match parse_attr(tmp_attributes.join(",")) {
                                        Ok(t) => (t.bits, t.align, t.global),
                                        Err(e) => {
                                            errors.push(e);
                                            continue;
                                        }
                                    };
                                ast.labels.push(Label {
                                    name: inside_label.1,
                                    inst: instructions,
                                    visibility: if global {
                                        Visibility::Global
                                    } else {
                                        Visibility::Local
                                    },
                                    bits: if matches!(bits, 16 | 32 | 64) {
                                        bits
                                    } else {
                                        ast.bits.unwrap_or(16)
                                    },
                                    align,
                                });
                                instructions = Vec::new();
                                tmp_attributes = Vec::new();
                                inside_label = (false, EMPTY_STRING.to_string())
                            }
                            if !s.is_empty() {
                                tmp_attributes.push(s)
                            }
                        }
                        ASTNode::Include(p) => ast.includes.push(p),
                        ASTNode::MathEval(name, value) => ast.math.push((name, value)),
                        ASTNode::Label(lbl) => {
                            if !instructions.is_empty() {
                                let (bits, align, global) =
                                    match parse_attr(tmp_attributes.join(",")) {
                                        Ok(t) => (t.bits, t.align, t.global),
                                        Err(e) => {
                                            errors.push(e);
                                            continue;
                                        }
                                    };
                                ast.labels.push(Label {
                                    name: inside_label.1,
                                    inst: instructions,
                                    visibility: if global {
                                        Visibility::Global
                                    } else {
                                        Visibility::Local
                                    },
                                    bits: if matches!(bits, 16 | 32 | 64) {
                                        bits
                                    } else {
                                        ast.bits.unwrap_or(16)
                                    },
                                    align,
                                });
                                instructions = Vec::new();
                                tmp_attributes = Vec::new();
                            }
                            inside_label = (true, lbl)
                        }
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
            let (bits, align, global) = match parse_attr(tmp_attributes.join(",")) {
                Ok(t) => (t.bits, t.align, t.global),
                Err(e) => {
                    errors.push(e);
                    (0, 0, false)
                }
            };
            ast.labels.push(Label {
                name: inside_label.1,
                inst: instructions,
                visibility: if global {
                    Visibility::Global
                } else {
                    Visibility::Local
                },
                bits: if matches!(bits, 16 | 32 | 64) {
                    bits
                } else {
                    ast.bits.unwrap_or(16)
                },
                align,
            });
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(ast)
        }
    }
}

fn split_str_into_vec(str: &str) -> Vec<String> {
    let mut strs = Vec::new();
    let mut buf = Vec::new();
    for b in str.as_bytes() {
        if b != &b',' {
            buf.push(*b);
        } else if b == &b' ' {
            continue;
        } else {
            strs.push(String::from_utf8(buf).expect("Code should be encoded in UTF-8"));
            buf = Vec::new();
        }
    }
    if !buf.is_empty() {
        strs.push(String::from_utf8(buf).expect("Code should be encoded in UTF-8"));
    }
    strs
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
struct TmpLabelAttr {
    align: u16,
    bits: u8,
    global: bool,
}

fn parse_attr(attr: String) -> Result<TmpLabelAttr, RASMError> {
    if attr.is_empty() {
        return Ok(TmpLabelAttr::default());
    }
    let mut attrs = TmpLabelAttr::default();
    let args = split_str_into_vec(&attr);
    for a in args {
        if let Some((key, val)) = a.split_once('=') {
            match key {
                "visibility" => {
                    if val == "global" {
                        attrs.global = true;
                    } else if val == "local" {
                        attrs.global = false;
                    } else {
                        return Err(RASMError::no_tip(None, Some("Tried to assign label visibility attribute; expected either \"global\" or \"local\", found unknown")));
                    }
                }
                "align" => {
                    if let Ok(n) = val.parse::<u16>() {
                        attrs.align = n;
                    } else {
                        return Err(RASMError::no_tip(None, Some("Tried to assign label align attribute; expected a unsigned 16-bit integer, found unknown")));
                    }
                }
                "bits" => {
                    if let Ok(n) = val.parse::<u8>() {
                        attrs.bits = n;
                    } else {
                        return Err(RASMError::no_tip(None, Some("Tried to assign label bits attribute; expected a unsigned 8-bit integer, found unknown")));
                    }
                }
                _ => {
                    return Err(RASMError::no_tip(
                        None,
                        Some(format!("Unknown attribute: {key}")),
                    ))
                }
            }
        } else {
            return Err(RASMError::no_tip(
                None,
                Some(format!("Unknown attribute: {a}")),
            ));
        }
    }
    Ok(attrs)
}
