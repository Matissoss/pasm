// rasmx86_64 - src/shr/kwd.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use std::{cmp::Ordering, str::FromStr};

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Keyword {
    Word,
    Byte,
    Bits,
    Any,
    Qword,
    Dword,
    Xword,
    Yword,
    Entry,
    Global,
    Extern,
    Include,

    // sections
    Section,
    Align,
    Exec,
    Write,
    Alloc,

    // symbol referencing
    Deref,
    Ref,

    Math,
}

// keyword is equal
#[inline(always)]
fn kwd_ie(
    str: &[u8],
    cmp: &'static [u8],
    start: usize,
    end: usize,
    kwd: Keyword,
) -> Result<Keyword, ()> {
    for idx in start..end {
        let res = str[idx].cmp(&cmp[idx]);
        if res != Ordering::Equal {
            return Err(());
        }
    }
    Ok(kwd)
}

impl FromStr for Keyword {
    type Err = ();
    fn from_str(kwd_str: &str) -> Result<Self, <Self as FromStr>::Err> {
        let kwd_raw = kwd_str.as_bytes();
        let kwd = kwd_raw;
        match kwd_raw.len() {
            // experimental
            3 => match kwd_raw[0] {
                b'a' => kwd_ie(kwd, b"any", 1, 2, Keyword::Any),
                b'r' => kwd_ie(kwd, b"ref", 1, 2, Keyword::Ref),
                _ => Err(()),
            },

            4 => match kwd_raw[0] as char {
                'e' => kwd_ie(kwd, b"exec", 1, 3, Keyword::Exec),
                'm' => kwd_ie(kwd, b"math", 1, 3, Keyword::Math),
                'b' => match kwd_raw[1] as char {
                    'y' => kwd_ie(kwd, b"byte", 2, 3, Keyword::Byte),
                    'i' => kwd_ie(kwd, b"bits", 2, 3, Keyword::Bits),
                    _ => Err(()),
                },
                'w' => kwd_ie(kwd, b"word", 1, 3, Keyword::Word),
                _ => Err(()),
            },
            5 => match kwd_raw[0] as char {
                'a' => match kwd_raw[1] as char {
                    'l' => match kwd_raw[2] as char {
                        'i' => kwd_ie(kwd, b"align", 3, 4, Keyword::Align),
                        'l' => kwd_ie(kwd, b"alloc", 3, 4, Keyword::Alloc),
                        _ => Err(()),
                    },
                    _ => Err(()),
                },
                'w' => kwd_ie(kwd, b"write", 1, 4, Keyword::Write),
                'x' => kwd_ie(kwd, b"xword", 1, 4, Keyword::Xword),
                'y' => kwd_ie(kwd, b"yword", 1, 4, Keyword::Yword),
                'q' => kwd_ie(kwd, b"qword", 1, 4, Keyword::Qword),
                'd' => match kwd_raw[1] { 
                    b'w' => kwd_ie(kwd, b"dword", 1, 4, Keyword::Dword),
                    b'e' => kwd_ie(kwd, b"deref", 1, 4, Keyword::Deref),
                    _    => Err(()),
                },
                'e' => kwd_ie(kwd, b"entry", 1, 4, Keyword::Entry),
                _ => Err(()),
            },
            6 => match kwd_raw[0] as char {
                'g' => kwd_ie(kwd, b"global", 1, 5, Keyword::Global),
                'e' => kwd_ie(kwd, b"extern", 1, 5, Keyword::Extern),
                _ => Err(()),
            },
            7 => match kwd_raw[0] as char {
                'i' => kwd_ie(kwd, b"include", 1, 6, Keyword::Include),
                's' => kwd_ie(kwd, b"section", 1, 6, Keyword::Section),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Keyword {
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}
