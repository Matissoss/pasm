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
            3 => kwd_ie(kwd, b"any", 0, 2, Keyword::Any),

            4 => match kwd_raw[0] as char {
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
                'x' => kwd_ie(kwd, b"xword", 1, 4, Keyword::Xword),
                'y' => kwd_ie(kwd, b"yword", 1, 4, Keyword::Yword),
                'q' => kwd_ie(kwd, b"qword", 1, 4, Keyword::Qword),
                'd' => kwd_ie(kwd, b"dword", 1, 4, Keyword::Dword),
                'e' => kwd_ie(kwd, b"entry", 1, 4, Keyword::Entry),
                _ => Err(()),
            },
            6 => match kwd_raw[0] as char {
                'g' => kwd_ie(kwd, b"global", 1, 5, Keyword::Global),
                'e' => kwd_ie(kwd, b"extern", 1, 5, Keyword::Extern),
                _ => Err(()),
            },
            7 => kwd_ie(kwd, b"include", 1, 6, Keyword::Include),
            _ => Err(()),
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Keyword {
    fn to_string(&self) -> String {
        match self {
            Self::Include => String::from("include"),
            Self::Math => String::from("math"),
            Self::Qword => String::from("qword"),
            Self::Any => String::from("any"),
            Self::Dword => String::from("dword"),
            Self::Word => String::from("word"),
            Self::Byte => String::from("byte"),
            Self::Entry => String::from("entry"),
            Self::Global => String::from("global"),
            Self::Extern => String::from("extern"),
            Self::Bits => String::from("bits"),
            Self::Xword => String::from("xword"),
            Self::Yword => String::from("yword"),
        }
    }
}
