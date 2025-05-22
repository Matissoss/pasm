// rasmx86_64 - src/shr/size.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::kwd::Keyword;

use std::cmp::Ordering;
use std::fmt::{Display, Error as FmtError, Formatter};

#[derive(Debug, Copy, Clone, Default, Eq)]
pub enum Size {
    Byte,
    Word,
    Dword,
    Qword,
    Xword, // xmm0-15
    Yword, // ymm0-15
    #[default]
    Unknown,
    Any,
}

impl From<Size> for u8 {
    fn from(size: Size) -> u8 {
        match size {
            Size::Any | Size::Unknown => 0,
            Size::Byte => 1,
            Size::Word => 2,
            Size::Dword => 4,
            Size::Qword => 8,
            Size::Xword => 16,
            Size::Yword => 32,
        }
    }
}

impl TryFrom<u16> for Size {
    type Error = ();
    fn try_from(val: u16) -> Result<Self, <Self as TryFrom<u16>>::Error> {
        match val {
            1 => Ok(Self::Byte),
            2 => Ok(Self::Word),
            4 => Ok(Self::Dword),
            8 => Ok(Self::Qword),
            16 => Ok(Self::Xword),
            32 => Ok(Self::Yword),
            _ => Err(()),
        }
    }
}

impl Display for Size {
    fn fmt(&self, form: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self {
            Self::Byte => write!(form, "byte"),
            Self::Word => write!(form, "word"),
            Self::Dword => write!(form, "dword"),
            Self::Qword => write!(form, "qword"),
            Self::Xword => write!(form, "xword"),
            Self::Yword => write!(form, "yword"),
            Self::Any => write!(form, "{{any}}"),
            Self::Unknown => write!(form, "{{unknown}}"),
        }
    }
}

impl PartialOrd for Size {
    fn partial_cmp(&self, oth: &Size) -> Option<Ordering> {
        if self == &Size::Any || oth == &Size::Any {
            return Some(Ordering::Equal);
        }

        let s = *self as u16;
        let o = *oth as u16;

        Some(s.cmp(&o))
    }
}

impl PartialEq for Size {
    fn eq(&self, oth: &Size) -> bool {
        if *self as u8 == Size::Any as u8 || *oth as u8 == Size::Any as u8 {
            return true;
        }
        let s = *self as u8;
        let o = *oth as u8;

        s == o
    }
}

impl TryFrom<Keyword> for Size {
    type Error = ();
    fn try_from(kwd: Keyword) -> Result<Self, <Self as TryFrom<Keyword>>::Error> {
        match kwd {
            Keyword::Byte => Ok(Self::Byte),
            Keyword::Word => Ok(Self::Word),
            Keyword::Dword => Ok(Self::Dword),
            Keyword::Qword => Ok(Self::Qword),
            Keyword::Xword => Ok(Self::Xword),
            Keyword::Yword => Ok(Self::Yword),
            Keyword::Any => Ok(Self::Any),
            _ => Err(()),
        }
    }
}
