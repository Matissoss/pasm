// pasm - src/pre/par.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    pre::mer::{BodyNode, MergerToken, RootNode},
    shr::{
        ast::AST, error::Error, label::Label, num::Number, section::Section, visibility::Visibility,
    },
};

use std::{mem::ManuallyDrop, path::PathBuf};

pub fn par_attrs(label: &mut Label, attrs: &[&str]) -> Result<(), Error> {
    for a in attrs {
        for a in crate::utils::split_str_ref(a.as_bytes(), ',') {
            let key = a;
            let (key, val) = if let Some((key, val)) = key.split_once('=') {
                (key, Some(val))
            } else {
                (key, None)
            };
            match key {
            "global"|"public" => label.attributes.set_visibility(Visibility::Public),
            "protected" => label.attributes.set_visibility(Visibility::Protected),
            "weak" => label.attributes.set_visibility(Visibility::Weak),
            "align" => if let Some(num) = val {
                if let Some(num) = Number::from_str(num) {
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
                if let Some(num) = Number::from_str(num) {
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
    }
    Ok(())
}

#[derive(PartialEq, Default)]
pub struct ParserStatus<'a> {
    pub inroot: bool,
    pub started: bool,

    pub label: Label<'a>,
    pub attrs: Vec<&'a str>,
    pub section: Section<'a>,
}

// we have to use raw pointers, because if we used &mut instead, then rust would rip us apart :D
// also *mut Error, because it is cheaper than Option<Error>;
// you have to free *mut Error yourself (it is ManuallyDrop)
pub fn par<'a>(
    ast: *mut AST<'a>,
    node: MergerToken<'a>,
    status: *mut ParserStatus<'a>,
    lnum: usize,
) -> *mut Error {
    let ast = unsafe { &mut *ast };
    let status = unsafe { &mut *status };
    if let MergerToken::Body(b) = node {
        status.inroot = false;
        let body_node = b;
        match body_node {
            BodyNode::Exec => status.section.attributes.set_exec(true),
            BodyNode::Alloc => status.section.attributes.set_alloc(true),
            BodyNode::Write => status.section.attributes.set_write(true),
            BodyNode::Bits(b) => {
                status.section.bits = b;
            }
            BodyNode::Align(a) => {
                status.section.align = a;
            }
            BodyNode::Attributes(a) => {
                status.attrs.push(a);
            }
            BodyNode::Instruction(i) => {
                status.label.content.push(i);
            }
            BodyNode::Label(l) => {
                if !status.label.name.is_empty() {
                    status
                        .section
                        .content
                        .push(std::mem::take(&mut status.label));
                    status.label = l;
                    status
                        .label
                        .attributes
                        .set_bits(ast.default_bits.unwrap_or(16));
                    if let Err(er) = par_attrs(&mut status.label, &status.attrs) {
                        let ptr = std::ptr::from_mut(&mut *ManuallyDrop::new(er));
                        return ptr;
                    }
                } else {
                    status.label = l;
                    status
                        .label
                        .attributes
                        .set_bits(ast.default_bits.unwrap_or(16));
                    if let Err(er) = par_attrs(&mut status.label, &status.attrs) {
                        let ptr = std::ptr::from_mut(&mut *ManuallyDrop::new(er));
                        return ptr;
                    }
                }
            }
            BodyNode::Section(s) => {
                if !status.label.name.is_empty() {
                    status
                        .section
                        .content
                        .push(std::mem::take(&mut status.label));
                    status.label = Label::default();
                }
                if status.started && status.section != Section::default() {
                    ast.sections.push(std::mem::take(&mut status.section));
                    status.section = s;
                } else {
                    status.section = s;
                    status.started = true;
                }
            }
        }
    } else if let MergerToken::Root(r) = node {
        if !status.inroot {
            let er = Error::new_wline("you tried to use root node outside of root", 21, lnum);
            let ptr = std::ptr::from_mut(&mut *ManuallyDrop::new(er));
            return ptr;
        }
        match r {
            RootNode::Format(f) => {
                if ast.format.is_some() {
                    let er =
                        Error::new_wline("you tried to redeclare format multiple times", 21, lnum);
                    let ptr = std::ptr::from_mut(&mut *ManuallyDrop::new(er));
                    return ptr;
                } else {
                    ast.format = Some(f);
                }
            }
            RootNode::Bits(b) => {
                if ast.default_bits.is_some() && ast.default_bits == Some(b) {
                    let er = Error::new_wline(
                        "you tried to redeclare default bits multiple times",
                        21,
                        lnum,
                    );
                    let ptr = std::ptr::from_mut(&mut *ManuallyDrop::new(er));
                    return ptr;
                } else {
                    ast.default_bits = Some(b);
                }
            }
            RootNode::Output(o) => {
                if ast.default_output.is_some()
                    && ast.default_output == Some(PathBuf::from(o.to_string()))
                {
                    let er = Error::new_wline(
                        "you tried to redeclare default output path multiple times",
                        21,
                        lnum,
                    );
                    let ptr = std::ptr::from_mut(&mut *ManuallyDrop::new(er));
                    return ptr;
                } else {
                    ast.default_output = Some(PathBuf::from(o.to_string()));
                }
            }
            RootNode::Define(name, value) => {
                if ast.defines.insert(name, value).is_some() {
                    let er =
                        Error::new_wline("tried to redeclare same define multiple times", 21, lnum);
                    let ptr = std::ptr::from_mut(&mut *ManuallyDrop::new(er));
                    return ptr;
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
    std::ptr::null_mut()
}
