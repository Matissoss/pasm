// rasmx86_64 - src/shr/kwd.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::conf::FAST_MODE;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Keyword {
    Word,
    Byte,
    Bits,
    Qword,
    Dword,
    Xword,
    Const,
    Ronly,
    Entry,
    Global,
    Uninit,
    Extern,
}

// keyword is equal
#[inline(always)]
fn kwd_ie(kwd_raw: &str, kwd_dest: &str, kwd: Keyword) -> Result<Keyword, ()> {
    if FAST_MODE {
        Ok(kwd)
    } else {
        if kwd_raw == kwd_dest {
            Ok(kwd)
        } else {
            Err(())
        }
    }
}

impl FromStr for Keyword {
    type Err = ();
    fn from_str(kwd_str: &str) -> Result<Self, <Self as FromStr>::Err> {
        let kwd_raw = kwd_str.as_bytes();
        match kwd_raw.len() {
            4 => match kwd_raw[1] as char {
                'y' => kwd_ie(kwd_str, "byte", Keyword::Byte),
                'o' => kwd_ie(kwd_str, "word", Keyword::Word),
                'i' => kwd_ie(kwd_str, "bits", Keyword::Bits),
                _ => Err(()),
            },
            5 => match kwd_raw[0] as char {
                'x' => kwd_ie(kwd_str, "xword", Keyword::Xword),
                'q' => kwd_ie(kwd_str, "qword", Keyword::Qword),
                'd' => kwd_ie(kwd_str, "dword", Keyword::Dword),
                'e' => kwd_ie(kwd_str, "entry", Keyword::Entry),
                'c' => kwd_ie(kwd_str, "const", Keyword::Const),
                'r' => kwd_ie(kwd_str, "ronly", Keyword::Ronly),
                _ => Err(()),
            },
            6 => match kwd_raw[0] as char {
                'u' => kwd_ie(kwd_str, "uninit", Keyword::Uninit),
                'g' => kwd_ie(kwd_str, "global", Keyword::Global),
                'e' => kwd_ie(kwd_str, "extern", Keyword::Extern),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Keyword {
    fn to_string(&self) -> String {
        match self {
            Self::Qword => String::from("qword"),
            Self::Dword => String::from("dword"),
            Self::Word => String::from("word"),
            Self::Byte => String::from("byte"),
            Self::Uninit => String::from("uninit"),
            Self::Ronly => String::from("ronly"),
            Self::Const => String::from("data"),
            Self::Entry => String::from("entry"),
            Self::Global => String::from("global"),
            Self::Extern => String::from("extern"),
            Self::Bits => String::from("bits"),
            Self::Xword => String::from("xword"),
        }
    }
}
