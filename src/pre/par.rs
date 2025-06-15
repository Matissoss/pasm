// rasmx86_64 - src/pre/par.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

const EMPTY_STRING: &str = "";
use crate::shr::{
    ast::{ASTNode, Instruction, Label, AST},
    error::RASMError,
    section::Section,
    symbol::Visibility,
};

pub struct Parser;

const PAR_INST_CAP: usize = 16;

type Error = RASMError;
type LexTree = Vec<Result<(ASTNode, usize), RASMError>>;
impl Parser {
    pub fn build_tree(list: LexTree) -> Result<AST, Vec<RASMError>> {
        let mut errors: Vec<RASMError> = Vec::new();
        let mut ast = AST::default();
        let mut tmp_attributes: Vec<String> = Vec::with_capacity(4);
        let mut inside_label: (bool, String) = (false, String::new());
        let mut instructions: Vec<Instruction> = Vec::with_capacity(PAR_INST_CAP);
        let mut section_idx: usize = 0;
        let mut inside_section = Section::default();
        let mut labels = Vec::new();
        for node in list {
            match node {
                Err(error) => errors.push(error),
                Ok(node) => {
                    match node.0 {
                        ASTNode::Write => {
                            if inside_label.0 {
                                errors.push(Error::no_tip(
                                    Some(node.1),
                                    Some("Align keyword can only be used inside sections."),
                                ));
                            } else {
                                inside_section.attributes.set_write(true);
                            }
                        }
                        ASTNode::Exec => {
                            if inside_label.0 {
                                errors.push(Error::no_tip(
                                    Some(node.1),
                                    Some("Align keyword can only be used inside sections."),
                                ));
                            } else {
                                inside_section.attributes.set_exec(true);
                            }
                        }
                        ASTNode::Alloc => {
                            if inside_label.0 {
                                errors.push(Error::no_tip(
                                    Some(node.1),
                                    Some("Align keyword can only be used inside sections."),
                                ));
                            } else {
                                inside_section.attributes.set_alloc(true);
                            }
                        }
                        ASTNode::Align(a) => {
                            if inside_label.0 {
                                errors.push(Error::no_tip(
                                    Some(node.1),
                                    Some("Align keyword can only be used inside sections."),
                                ));
                            } else {
                                inside_section.align = a;
                            }
                        }
                        ASTNode::Section(s) => {
                            if inside_section == Section::default() {
                                inside_section = Section::default();
                                inside_section.bits = 16;
                                inside_section.name = s;
                                inside_label = (false, EMPTY_STRING.to_string());
                                continue;
                            }
                            if !instructions.is_empty() {
                                if let Err(err) = collect_label(
                                    &mut labels,
                                    tmp_attributes.join(","),
                                    instructions,
                                    inside_label.1,
                                    inside_section.bits,
                                    section_idx,
                                ) {
                                    errors.push(err);
                                }
                            }
                            inside_section.content = labels;
                            labels = Vec::new();
                            ast.sections.push(inside_section);
                            inside_section = Section::default();
                            inside_section.bits = 16;
                            inside_section.name = s;
                            section_idx += 1;
                            tmp_attributes.clear();
                            instructions = Vec::with_capacity(PAR_INST_CAP);
                            inside_label = (false, EMPTY_STRING.to_string());
                        }
                        ASTNode::Attributes(s) => {
                            if !instructions.is_empty() {
                                if let Err(err) = collect_label(
                                    &mut labels,
                                    tmp_attributes.join(","),
                                    instructions,
                                    inside_label.1,
                                    inside_section.bits,
                                    section_idx,
                                ) {
                                    errors.push(err);
                                }
                                instructions = Vec::with_capacity(PAR_INST_CAP);
                                tmp_attributes.clear();
                                inside_label = (false, EMPTY_STRING.to_string());
                            }
                            if !s.is_empty() {
                                tmp_attributes.push(s)
                            }
                        }
                        ASTNode::Include(p) => ast.includes.push(p),
                        ASTNode::MathEval(name, value) => ast.math.push((name, value)),
                        ASTNode::Label(lbl) => {
                            if !instructions.is_empty() {
                                if let Err(err) = collect_label(
                                    &mut labels,
                                    tmp_attributes.join(","),
                                    instructions,
                                    inside_label.1,
                                    inside_section.bits,
                                    section_idx,
                                ) {
                                    errors.push(err);
                                }
                                instructions = Vec::with_capacity(PAR_INST_CAP);
                                tmp_attributes.clear();
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
                        ASTNode::Bits(bits_new) => {
                            if (inside_label.0, inside_label.1.as_str()) == (false, EMPTY_STRING) {
                                match bits_new {
                                    16 | 32 | 64 => inside_section.bits = bits_new,
                                    n => errors.push(RASMError::no_tip(
                                        Some(node.1),
                                        Some(format!(
                                            "Invalid bits specifier; expected 16, 32, 64, found {}",
                                            n
                                        )),
                                    )),
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
            if let Err(err) = collect_label(
                &mut labels,
                tmp_attributes.join(","),
                instructions,
                inside_label.1,
                inside_section.bits,
                section_idx,
            ) {
                errors.push(err);
            }
        }
        if inside_section != Section::default() {
            inside_section.content = labels;
            ast.sections.push(inside_section);
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(ast)
        }
    }
}

fn split_str_into_vec(str: &str) -> Vec<String> {
    let mut strs = Vec::with_capacity(8);
    let mut buf = Vec::with_capacity(24);
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

fn collect_label(
    vec: &mut Vec<Label>,
    attrs: String,
    inst: Vec<Instruction>,
    name: String,
    defbits: u8,
    secidx: usize,
) -> Result<(), Error> {
    let (bits, align, global) = match parse_attr(attrs) {
        Ok(t) => (t.bits, t.align, t.global),
        Err(e) => return Err(e),
    };
    vec.push(Label {
        name,
        inst,
        visibility: if global {
            Visibility::Global
        } else {
            Visibility::Local
        },
        bits: if matches!(bits, 16 | 32 | 64) {
            bits
        } else {
            defbits
        },
        shidx: secidx,
        align,
    });
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn section_par_test() {
        use crate::shr::*;
        use section::SectionAttributes;
        let nodes = vec![
            Ok((ASTNode::Section(".text".to_string()), 0)),
            Ok((ASTNode::Align(16), 0)),
            Ok((ASTNode::Bits(64), 0)),
            Ok((ASTNode::Label("test".to_string()), 0)),
            Ok((
                ASTNode::Ins(Instruction {
                    oprs: [None, None, None, None, None],
                    addt: None,
                    line: 0,
                    mnem: ins::Mnemonic::__LAST,
                }),
                0,
            )),
            Ok((ASTNode::Label("tesy".to_string()), 0)),
            Ok((
                ASTNode::Ins(Instruction {
                    oprs: [None, None, None, None, None],
                    addt: None,
                    line: 0,
                    mnem: ins::Mnemonic::__LAST,
                }),
                0,
            )),
            Ok((ASTNode::Section(".text1".to_string()), 0)),
            Ok((ASTNode::Align(16), 0)),
            Ok((ASTNode::Bits(64), 0)),
            Ok((ASTNode::Label("test".to_string()), 0)),
            Ok((
                ASTNode::Ins(Instruction {
                    oprs: [None, None, None, None, None],
                    addt: None,
                    line: 0,
                    mnem: ins::Mnemonic::__LAST,
                }),
                0,
            )),
            Ok((ASTNode::Label("tesy".to_string()), 0)),
            Ok((
                ASTNode::Ins(Instruction {
                    oprs: [None, None, None, None, None],
                    addt: None,
                    line: 0,
                    mnem: ins::Mnemonic::__LAST,
                }),
                0,
            )),
        ];
        let ast = Parser::build_tree(nodes);
        assert_eq!(true, ast.is_ok());
        assert_eq!(
            ast.unwrap().sections,
            vec![
            Section {
                name: String::from(".text"),
                align: 16,
                offset: 0,
                size: 0,
                attributes: SectionAttributes::new(),
                content: vec![
                    Label {
                        name: String::from("test"),
                        align: 0,
                        visibility: symbol::Visibility::Local,
                        bits: 64,
                        inst: vec![Instruction {
                            oprs: [None, None, None, None, None],
                            addt: None,
                            line: 0,
                            mnem: ins::Mnemonic::__LAST,
                        }],
                        shidx: 0,
                    },
                    Label {
                        name: String::from("tesy"),
                        align: 0,
                        visibility: symbol::Visibility::Local,
                        bits: 64,
                        inst: vec![Instruction {
                            oprs: [None, None, None, None, None],
                            addt: None,
                            line: 0,
                            mnem: ins::Mnemonic::__LAST,
                        }],
                        shidx: 0,
                    },
                ],
                bits: 64
            },
            Section {
                name: String::from(".text1"),
                align: 16,
                offset: 0,
                size: 0,
                attributes: SectionAttributes::new(),
                content: vec![
                    Label {
                        name: String::from("test"),
                        align: 0,
                        visibility: symbol::Visibility::Local,
                        bits: 64,
                        inst: vec![Instruction {
                            oprs: [None, None, None, None, None],
                            addt: None,
                            line: 0,
                            mnem: ins::Mnemonic::__LAST,
                        }],
                        shidx: 1,
                    },
                    Label {
                        name: String::from("tesy"),
                        align: 0,
                        visibility: symbol::Visibility::Local,
                        bits: 64,
                        inst: vec![Instruction {
                            oprs: [None, None, None, None, None],
                            addt: None,
                            line: 0,
                            mnem: ins::Mnemonic::__LAST,
                        }],
                        shidx: 1,
                    },
                ],
                bits: 64
            },
        ]
        );
    }
}
