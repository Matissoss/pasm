// pasm - src/pre/par.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

const EMPTY_STRING: &str = "";
use crate::shr::{
    ast::{ASTNode, Instruction, Label, AST},
    error::RASMError,
    section::Section,
    symbol::SymbolType,
};

use crate::RString;

type Error = RASMError;
type LexTree = Vec<Result<(ASTNode, usize), RASMError>>;

pub fn ast(list: LexTree) -> Result<AST, Vec<RASMError>> {
    let empty: RString = EMPTY_STRING.into();
    let mut errors: Vec<RASMError> = Vec::new();
    let mut ast = AST::default();
    let mut tmp_attributes: Vec<RString> = Vec::with_capacity(4);
    let mut inside_label: (bool, RString) = (false, RString::from(""));
    let mut instructions: Vec<Instruction> = Vec::new();
    let mut section_idx: u16 = 0;
    let mut inside_section = Section::default();
    let mut labels = Vec::new();
    for node in list {
        match node {
            Err(error) => errors.push(error),
            Ok(node) => match node.0 {
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
                        inside_label = (false, empty.clone());
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
                    instructions = Vec::new();
                    inside_label = (false, empty.clone());
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
                        instructions = Vec::new();
                        tmp_attributes.clear();
                        inside_label = (false, empty.clone());
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
                        instructions = Vec::new();
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
                    if (inside_label.0, &inside_label.1) == (false, &empty) {
                        ast.externs.push(extrn);
                    } else {
                        errors.push(RASMError::no_tip(
                            Some(node.1),
                            Some("Externs can be only declared outside of labels, not inside of"),
                        ));
                    }
                }
                ASTNode::Entry(entry) => {
                    if (inside_label.0, &inside_label.1) == (false, &empty) {
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
                    if (inside_label.0, &inside_label.1) == (false, &empty) {
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
            },
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
    } else if !labels.is_empty() {
        inside_section.bits = 16;
        inside_section.name = String::from(".rasm.default").into();
        inside_section.content = labels;
        ast.sections.push(inside_section);
    }

    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok(ast)
    }
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
struct TmpLabelAttr {
    align: u16,
    bits: u8,
    global: bool,
    stype: SymbolType,
}

fn collect_label(
    vec: &mut Vec<Label>,
    attrs: String,
    inst: Vec<Instruction>,
    name: RString,
    defbits: u8,
    secidx: u16,
) -> Result<(), Error> {
    let (bits, align, global, stype) = match parse_attr(attrs.to_string()) {
        Ok(t) => (t.bits, t.align, t.global, t.stype),
        Err(e) => return Err(e),
    };
    vec.push(Label {
        name,
        inst,
        meta: (global as u8) << 7
            | if matches!(bits, 16 | 32 | 64) {
                bits >> 4
            } else {
                defbits >> 4
            },
        shidx: secidx,
        align,
        stype,
    });
    Ok(())
}

fn parse_attr(attr: String) -> Result<TmpLabelAttr, RASMError> {
    if attr.is_empty() {
        return Ok(TmpLabelAttr::default());
    }
    let mut attrs = TmpLabelAttr::default();
    let args = crate::utils::split_str_owned(&attr, ',');
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
                "type" => match val {
                    "func" => attrs.stype = SymbolType::Func,
                    "objc" => attrs.stype = SymbolType::Object,
                    "sect" => attrs.stype = SymbolType::Section,
                    "file" => attrs.stype = SymbolType::File,
                    "none" => attrs.stype = SymbolType::NoType,
                    _ => {
                        return Err(RASMError::msg(format!(
                            "Tried to use unknown type in attributes: \"{val}\""
                        )))
                    }
                },
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
