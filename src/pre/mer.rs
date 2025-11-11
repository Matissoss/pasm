// pasm - src/pre/mer.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    pre::tok::Token,
    shr::{
        ast,
        dir,
        ins,
        error::Error,
    },
};

pub enum MerToken<'a> {
    Unknown(&'a str),
    Operand(ast::Operand<'a>),
    Directive(dir::Directive),
    Mnemonic(ins::Mnemonic),
    Delimeter(Box<[Vec<MerToken<'a>>]>),
    LabelDec(&'a str),
    SectionDec(&'a str),
    Colon,
    Comma,
    Dot,
}

// TODO: replace with SmallVec
pub fn mert<'a>(line_raw: &'a str, line_num: usize, toks: Vec<Token>) -> Result<Vec<MerToken<'a>>, Error> {
    let mut mertoks: Vec<MerToken<'a>> = Vec::new();
    let mut tmp_buf: Vec<MerToken<'a>> = Vec::new();
    // for situations like: (delimeter1 (delimeter2))
    let mut delimeter_recursion: Vec<Vec<MerToken<'a>>> = Vec::new();
    // for informing about unclosed delimeter
    // idx 0 is for string delimeters
    // idx 1 is for char delimeters
    // idx 2 is for ( delimeters (don't know name for it)
    let mut delimeter_status: [u16; 3] = [0; 3];
    for (_,t) in toks.into_iter().enumerate() { 
        match t {
            Token::CharDelimeter => {
                delimeter_status[1] = !((delimeter_status[1] != 0) as u16);
            }
            Token::StringDelimeter => {
                delimeter_status[0] = !((delimeter_status[0] != 0) as u16);
            }
            Token::Comma => {
                if tmp_buf.is_empty() {
                    mertoks.push(MerToken::Comma);
                    continue;
                }
                
            }
            Token::Dot => {
                tmp_buf.push(MerToken::Dot);
            },
            Token::String(slice_start, slice_end) => {
                let slice = unsafe { line_raw.get_unchecked((slice_start as usize)..(slice_end as usize)) };
                if let Some(MerToken::Dot|MerToken::Directive(dir::Directive::Section)) = tmp_buf.last() {
                    tmp_buf.pop();
                    tmp_buf.push(MerToken::SectionDec(slice));
                    continue;
                }
            }
            Token::Colon => {
                if let Some(MerToken::Unknown(_)) = tmp_buf.last() {
                    let tu = tmp_buf.pop().unwrap();
                    let tu = if let MerToken::Unknown(s) = tu {
                        s
                    } else {
                        panic!();
                    };
                    tmp_buf.push(MerToken::LabelDec(tu));
                } else {
                    tmp_buf.push(MerToken::Colon);
                }
            }
            Token::ClosureOpen => {
                delimeter_status[2] += 1;
                mertoks.extend(tmp_buf);
                tmp_buf = Vec::new();
            },
            Token::ClosureClose => {
                if delimeter_status[2] == 0 {
                    return Err(Error::new_wline("One (or more) too many closing delimeters.", 2, line_num));
                }
                delimeter_status[2] -= 1;
                delimeter_recursion.push(tmp_buf);
                tmp_buf = Vec::new();
                if delimeter_status[2] == 0 {
                    mertoks.push(MerToken::Delimeter(delimeter_recursion.into()));
                }
                delimeter_recursion = Vec::new();
            }
            Token::Semicolon => break,
        }
    }
    for (idx, ds) in delimeter_status.into_iter().enumerate() {
        if ds != 0 {
            let mut error = match idx {
                0 => Error::new("Unclosed \" delimeter.", 1),
                1 => Error::new("Unclosed \' delimeter.", 1),
                2 => Error::new("Unclosed ( delimeter.", 1),
                _ => Error::new("Internal error src/pre/mer.rs in mert", 0),
            };
            error.set_line(line_num);
        }
    }
    Ok(mertoks)
}
