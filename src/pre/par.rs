// pasm - src/pre/par.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    pre::mer::{BodyNodeEnum as BodyNode, MergerResult, RootNodeEnum as RootNode},
    shr::{
        ast::AST, error::RError as Error, label::Label, location::Location, num::Number,
        section::Section, visibility::Visibility,
    },
};

use std::path::PathBuf;

pub fn par_attrs(label: &mut Label, attrs: String) -> Result<(), Error> {
    for a in crate::utils::split_str_owned(&attrs, ',') {
        let key = a;
        let (key, val) = if let Some((key, val)) = key.split_once('=') {
            (key.to_string(), Some(val))
        } else {
            (key, None)
        };
        match key.as_str() {
            "global"|"public" => label.attributes.set_visibility(Visibility::Public),
            "protected" => label.attributes.set_visibility(Visibility::Protected),
            "weak" => label.attributes.set_visibility(Visibility::Weak),
            "align" => if let Some(num) = val {
                if let Ok(num) = Number::from_str(num) {
                    label.align = num.get_as_u64() as u16;
                } else {
                    return Err(Error::new("align parameter needs a number, not a string", 20));
                }
            } else {
                return Err(Error::new(format!("usage of unknown key-only attribute: \"{key}\""), 20));
            },
            "type" => return Err(Error::new(format!("external type declarations are forbidden, use inline label attribute{} instead",
                if let Some(val) = val {
                    format!(" like: {} {}", val, label.name)
                } else {
                    "".to_string()
                }
            ), 20)),
            "bits" => if let Some(num) = val {
                if let Ok(num) = Number::from_str(num) {
                    match num.get_as_u64() {
                        16 | 32 | 64 => label.attributes.set_bits(num.get_as_u64() as u8),
                        _ => return Err(Error::new("usage of unknown bits parameter: expected 16, 32 or 64", 20)),
                    }
                } else {
                    return Err(Error::new("bits parameter needs a number, not a string", 20));
                }
            } else {
                return Err(Error::new(format!("usage of unknown key-only attribute: \"{key}\""), 20));
            },
            _ => return Err(Error::new(format!("usage of unknown key-only attribute: \"{key}\""), 20)),
        }
    }
    Ok(())
}

pub fn par(mer: MergerResult) -> Result<AST, Vec<Error>> {
    let mut ast = AST::default();
    let (mut oloc, mut bloc, mut floc) = (
        Location::default(),
        Location::default(),
        Location::default(),
    );
    let mut errors = Vec::new();

    // setup root
    for root_node in mer.root {
        let location = root_node.location;
        match root_node.node {
            RootNode::Format(f) => {
                if ast.format.is_some() && ast.format.as_ref() == Some(&f) {
                    let mut er = Error::new_wline(
                        "you tried to redeclare format multiple times",
                        21,
                        location.line,
                    );
                    er.set_context(floc);
                    errors.push(er);
                    floc = Location::default();
                } else {
                    ast.format = Some(f);
                    floc = location;
                }
            }
            RootNode::Bits(b) => {
                if ast.default_bits.is_some() && ast.default_bits == Some(b) {
                    let mut er = Error::new_wline(
                        "you tried to redeclare default bits multiple times",
                        21,
                        location.line,
                    );
                    er.set_context(bloc);
                    bloc = Location::default();
                    errors.push(er);
                } else {
                    ast.default_bits = Some(b);
                    bloc = location;
                }
            }
            RootNode::Output(o) => {
                if ast.default_output.is_some()
                    && ast.default_output == Some(PathBuf::from(o.to_string()))
                {
                    let mut er = Error::new_wline(
                        "you tried to redeclare default output path multiple times",
                        21,
                        location.line,
                    );
                    er.set_context(oloc);
                    oloc = Location::default();
                    errors.push(er);
                } else {
                    ast.default_output = Some(PathBuf::from(o.to_string()));
                    oloc = location;
                }
            }
            RootNode::Define(name, value) => {
                if ast.defines.insert(name, value).is_some() {
                    let er = Error::new_wline(
                        "tried to redeclare same define multiple times",
                        21,
                        location.line,
                    );
                    errors.push(er);
                }
            }
            RootNode::Extern(e) => {
                ast.externs.push(e);
            }
            RootNode::Include(i) => {
                ast.includes.push(PathBuf::from(i.to_string()));
            }
        }
    }

    // setup body
    let mut started = false;
    let mut section = Section::default();
    let mut label = Label::default();
    let mut attrs = Vec::new();
    for body_node in mer.body {
        let body_node = body_node.node;
        match body_node {
            BodyNode::Exec => section.attributes.set_exec(true),
            BodyNode::Alloc => section.attributes.set_alloc(true),
            BodyNode::Write => section.attributes.set_write(true),
            BodyNode::Bits(b) => {
                section.bits = b;
            }
            BodyNode::Align(a) => {
                section.align = a;
            }
            BodyNode::Attributes(a) => {
                attrs.push(a);
            }
            BodyNode::Instruction(i) => {
                label.content.push(i);
            }
            BodyNode::Label(l) => {
                if label == Label::default() {
                    label = l;
                    label.attributes.set_bits(ast.default_bits.unwrap_or(16));
                    if !attrs.is_empty() {
                        if let Err(why) = par_attrs(&mut label, attrs.join(",")) {
                            errors.push(why);
                        }
                    }
                } else {
                    section.content.push(label);
                    label = l;
                    label.attributes.set_bits(ast.default_bits.unwrap_or(16));
                }
            }
            BodyNode::Section(s) => {
                if started && section != Section::default() {
                    ast.sections.push(section);
                    section = Section::default();
                    section.name = s;
                } else {
                    section = Section::default();
                    section.name = s;
                    started = true;
                }
            }
        }
    }

    if label != Label::default() {
        label.attributes.set_bits(ast.default_bits.unwrap_or(16));
        section.content.push(label);
    }

    if section != Section::default() {
        ast.sections.push(section);
    }

    if errors.is_empty() {
        Ok(ast)
    } else {
        Err(errors)
    }
}
