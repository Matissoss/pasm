// pasm - src/pre/par.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

const EMPTY_STRING: &str = "";
use crate::shr::{
    ast::{ASTNode, Instruction, Label, AST},
    error::RError as Error,
    section::Section,
    symbol::SymbolType,
};

use crate::RString;

type LexTree = Vec<Result<(ASTNode, usize), Error>>;

pub fn ast(list: LexTree) -> Result<AST, Vec<Error>> {
    let empty: RString = EMPTY_STRING.into();
    let mut errors: Vec<Error> = Vec::new();
    let mut ast = AST::default();
    let mut tmp_attributes: Vec<RString> = Vec::with_capacity(4);
    let mut inside_label: (bool, RString) = (false, RString::from(""));
    let mut inside_label_line: usize = 0;
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
                        let mut er =
                            Error::new("write directive can only be used inside sections", 8);
                        er.set_line(node.1);
                        errors.push(er);
                    } else {
                        inside_section.attributes.set_write(true);
                    }
                }
                ASTNode::Exec => {
                    if inside_label.0 {
                        let mut er =
                            Error::new("exec directive can only be used inside sections", 8);
                        er.set_line(node.1);
                        errors.push(er);
                    } else {
                        inside_section.attributes.set_exec(true);
                    }
                }
                ASTNode::Alloc => {
                    if inside_label.0 {
                        let mut er =
                            Error::new("alloc directive can only be used inside sections", 8);
                        er.set_line(node.1);
                        errors.push(er);
                    } else {
                        inside_section.attributes.set_alloc(true);
                    }
                }
                ASTNode::Align(a) => {
                    if inside_label.0 {
                        let mut er =
                            Error::new("align directive can only be used inside sections", 8);
                        er.set_line(node.1);
                        errors.push(er);
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
                            inside_label_line,
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
                            inside_label_line,
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
                ASTNode::Define(name, value) => {
                    if ast.defined.insert(name.clone(), value).is_some() {
                        let mut err =
                            Error::new(format!("you tried to redeclare symbol \"{name}\""), 4);
                        err.set_line(node.1);
                        errors.push(err);
                    }
                }
                ASTNode::Label(lbl) => {
                    if !instructions.is_empty() {
                        if let Err(err) = collect_label(
                            &mut labels,
                            tmp_attributes.join(","),
                            instructions,
                            inside_label.1,
                            inside_section.bits,
                            section_idx,
                            inside_label_line,
                        ) {
                            errors.push(err);
                        }
                        instructions = Vec::new();
                        tmp_attributes.clear();
                    }
                    inside_label_line = node.1;
                    inside_label = (true, lbl)
                }
                ASTNode::Ins(ins) => {
                    if !inside_label.0 {
                        let mut er = Error::new("instructions can only be used inside labels", 8);
                        er.set_line(node.1);
                        errors.push(er);
                    } else {
                        instructions.push(ins);
                    }
                }
                ASTNode::Extern(extrn) => {
                    if (inside_label.0, &inside_label.1) == (false, &empty) {
                        ast.externs.push(extrn);
                    } else {
                        let mut er =
                            Error::new("extern directive can only be used outside of labels", 8);
                        er.set_line(node.1);
                        errors.push(er);
                    }
                }
                ASTNode::Bits(bits_new) => {
                    if (inside_label.0, &inside_label.1) == (false, &empty) {
                        match bits_new {
                            16 | 32 | 64 => inside_section.bits = bits_new,
                            _ => {
                                let mut er = Error::new(
                                    "bits directive can only accept values: 16, 32 or 64",
                                    8,
                                );
                                er.set_line(node.1);
                                errors.push(er);
                            }
                        }
                    } else {
                        let mut er =
                            Error::new("bits directive can only be used outside of labels", 8);
                        er.set_line(node.1);
                        errors.push(er);
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
            inside_label_line,
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
    line: usize,
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
        line,
    });
    Ok(())
}

fn parse_attr(attr: String) -> Result<TmpLabelAttr, Error> {
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
                        return Err(Error::new(
                            "unexpected visibility attribute value: expected global or local",
                            8,
                        ));
                    }
                }
                "type" => match val {
                    "func" => attrs.stype = SymbolType::Func,
                    "objc" => attrs.stype = SymbolType::Object,
                    "sect" => attrs.stype = SymbolType::Section,
                    "file" => attrs.stype = SymbolType::File,
                    "none" => attrs.stype = SymbolType::NoType,
                    _ => return Err(Error::new(
                        "unexpected type attribute value: expected func, objc, sect, file or none",
                        8,
                    )),
                },
                "align" => {
                    if let Ok(n) = val.parse::<u16>() {
                        attrs.align = n;
                    } else {
                        return Err(Error::new(
                            "unexpected align attribute value: expected 16-bit unsigned integer",
                            8,
                        ));
                    }
                }
                "bits" => {
                    if let Ok(n) = val.parse::<u8>() {
                        attrs.bits = n;
                    } else {
                        return Err(Error::new(
                            "unexpected bits attribute value: expected 8-bit unsigned integer",
                            8,
                        ));
                    }
                }
                _ => {
                    return Err(Error::new(
                        format!("tried to use unknown attribute \"{key}\""),
                        8,
                    ))
                }
            }
        } else {
            return Err(Error::new(
                format!("tried to use unknown attribute \"{a}\""),
                8,
            ));
        }
    }
    Ok(attrs)
}

// parser new

#[allow(unused_imports)]
use crate::pre::mer::{BodyNode, MergerResult, RootNode};

pub fn par(_mer: MergerResult) -> Result<AST, Error> {
    todo!()
}
