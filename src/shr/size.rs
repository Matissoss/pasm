// rasmx86_64 - size.rs
// --------------------
// made by matissoss
// licensed under MPL

use std::fmt::{
    Formatter,
    Display,
    Error as FmtError
};
use std::cmp::Ordering;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum Size{
    Byte,
    Word,
    Dword,
    Qword,
    Xword, // xmm0-15
    Yword, // ymm0-15
    #[default]
    Unknown,
}

impl Into<u8> for Size{
    fn into(self) -> u8 {
        match self{
            Self::Unknown   => 0,
            Self::Byte      => 1,
            Self::Word      => 2,
            Self::Dword     => 4,
            Self::Qword     => 8,
            Self::Xword     => 16,
            Self::Yword     => 32,
        }
    }
}

impl TryFrom<u8> for Size{
    type Error = ();
    fn try_from(val: u8) -> Result<Self, <Self as TryFrom<u8>>::Error>{
        match val {
            1  => Ok(Self::Byte),
            2  => Ok(Self::Word),
            4  => Ok(Self::Dword),
            8  => Ok(Self::Qword),
            16 => Ok(Self::Xword),
            32 => Ok(Self::Yword),
            _ => Err(())
        }
    }
}

impl TryFrom<u16> for Size{
    type Error = ();
    fn try_from(val: u16) -> Result<Self, <Self as TryFrom<u16>>::Error>{
        match val {
            1  => Ok(Self::Byte),
            2  => Ok(Self::Word),
            4  => Ok(Self::Dword),
            8  => Ok(Self::Qword),
            16 => Ok(Self::Xword),
            32 => Ok(Self::Yword),
            _ => Err(())
        }
    }
}

impl Display for Size{
    fn fmt(&self, form: &mut Formatter<'_>) -> Result<(), FmtError>{
        match self{
            Self::Byte      => write!(form, "byte"),
            Self::Word      => write!(form, "word"),
            Self::Dword     => write!(form, "dword"),
            Self::Qword     => write!(form, "qword"),
            Self::Xword     => write!(form, "xword"),
            Self::Yword     => write!(form, "yword"),
            Self::Unknown   => write!(form, "{}unknown{}", '{', '}'),
        }
    }
}

impl PartialOrd for Size{
    fn partial_cmp(&self, oth: &Size) -> Option<Ordering>{
        let s = *self as u16;
        let o = *oth  as u16;

        if s < o {
            return Some(Ordering::Less);
        }
        else if s == o {
            return Some(Ordering::Equal);
        }
        else {
            return Some(Ordering::Greater);
        }
    }
}
