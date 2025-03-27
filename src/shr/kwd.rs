// rasmx86_64 - kwd.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use std::str::FromStr;
use crate::conf::FAST_MODE;

#[derive(Debug)]
pub enum Keyword{
    Qword,
    Dword,
     Word,
     Byte,
    Section,
    Global,
    Resd,
    Resw,
    Resq,
    Resb,
    Db,
    Dw,
    Dd,
    Dq,
}

// keyword is equal
// helper for FromStr for Keyword
#[inline(always)]
fn kwd_ie(kwd_raw: &str, kwd_dest: &str, kwd: Keyword) -> Result<Keyword, ()>{
    if FAST_MODE{
        return Ok(kwd);
    }
    else {
        if kwd_raw == kwd_dest {
            return Ok(kwd);
        }
        return Err(());
    }
}

impl FromStr for Keyword{
    type Err = ();
    fn from_str(kwd_str: &str) -> Result<Self, <Self as FromStr>::Err>{
        let kwd_raw = kwd_str.as_bytes();
        match kwd_raw.len() {
            0|1|3 => return Err(()),
            2 => {
                return match kwd_raw[1] as char {
                    'd' => kwd_ie(kwd_str, "dd", Keyword::Dd),
                    'b' => kwd_ie(kwd_str, "db", Keyword::Db),
                    'w' => kwd_ie(kwd_str, "dw", Keyword::Dw),
                    'q' => kwd_ie(kwd_str, "dq", Keyword::Dq),
                    _   => return Err(())
                };
            },
            4 => {
                return match kwd_raw[1] as char {
                    'e' => {
                        match kwd_raw[3] as char {
                            'w' => kwd_ie(kwd_str, "resw", Keyword::Resw),
                            'd' => kwd_ie(kwd_str, "resd", Keyword::Resd),
                            'q' => kwd_ie(kwd_str, "resq", Keyword::Resq),
                            'b' => kwd_ie(kwd_str, "resb", Keyword::Resb),
                            _   => return Err(())
                        }
                    },
                    'y' => kwd_ie(kwd_str, "byte", Keyword::Byte),
                    'o' => kwd_ie(kwd_str, "word", Keyword::Word),
                    _   => return Err(())
                };
            },
            5 => {
                return match kwd_raw[0] as char{
                    'q' => kwd_ie(kwd_str, "qword", Keyword::Qword),
                    'd' => kwd_ie(kwd_str, "dword", Keyword::Dword),
                    _   => return Err(())
                };
            },
            6 => return kwd_ie(kwd_str, "global", Keyword::Global),
            7 => return kwd_ie(kwd_str, "section", Keyword::Section),
            _ => return Err(())
        }
    }
}
