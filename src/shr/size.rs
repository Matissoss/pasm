// pasm - src/shr/size.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

use std::cmp::Ordering;
use std::fmt::{Display, Error as FmtError, Formatter};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Default, Eq)]
pub enum Size {
    Byte,
    Word,
    Dword,
    Qword,
    Xword,
    Yword,
    Zword,
    B80, // for x87 purposes
    #[default]
    Unknown,
    Any,
}

impl FromStr for Size {
    type Err = ();
    fn from_str(line: &str) -> Result<Self, <Self as FromStr>::Err> {
        if let Some(sz) = size_fromstr(line) {
            Ok(sz)
        } else {
            Err(())
        }
    }
}

pub fn size_fromstr(str: &str) -> Option<Size> {
    use Size::*;
    let r = str.as_bytes();
    match r.len() {
        3 => match r[0] {
            b'b' => match r[1] {
                b'8' => match r[2] {
                    b'0' => Some(B80),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        },
        4 => match r[0] {
            b'b' => match r[1] {
                b'y' => match r[2] {
                    b't' => match r[3] {
                        b'e' => Some(Byte),
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            },
            b'w' => match r[1] {
                b'o' => match r[2] {
                    b'r' => match r[3] {
                        b'd' => Some(Word),
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        },
        5 => match r[0] {
            b'q' => match r[1] {
                b'w' => match r[2] {
                    b'o' => match r[3] {
                        b'r' => match r[4] {
                            b'd' => Some(Qword),
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            },
            b'x' => match r[1] {
                b'w' => match r[2] {
                    b'o' => match r[3] {
                        b'r' => match r[4] {
                            b'd' => Some(Xword),
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            },
            b'y' => match r[1] {
                b'w' => match r[2] {
                    b'o' => match r[3] {
                        b'r' => match r[4] {
                            b'd' => Some(Yword),
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            },
            b'z' => match r[1] {
                b'w' => match r[2] {
                    b'o' => match r[3] {
                        b'r' => match r[4] {
                            b'd' => Some(Zword),
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            },
            b'd' => match r[1] {
                b'w' => match r[2] {
                    b'o' => match r[3] {
                        b'r' => match r[4] {
                            b'd' => Some(Dword),
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

impl Size {
    pub fn is_any(&self) -> bool {
        *self as u8 == Self::Any as u8
    }
    pub const fn se(&self) -> u8 {
        match self {
            Self::Unknown | Self::Any => 0b0000,
            Self::Byte => 0b0001,
            Self::Word => 0b0010,
            Self::Dword => 0b0011,
            Self::Qword => 0b0100,
            Self::Xword => 0b0101,
            Self::Yword => 0b0110,
            Self::Zword => 0b0111,
            Self::B80 => 0b1000,
        }
    }
    pub const fn de(key: u8) -> Self {
        match key {
            0b0000 => Size::Any,
            0b0001 => Size::Byte,
            0b0010 => Size::Word,
            0b0011 => Size::Dword,
            0b0100 => Size::Qword,
            0b0101 => Size::Xword,
            0b0110 => Size::Yword,
            0b0111 => Size::Zword,
            0b1000 => Size::B80,
            _ => Size::Unknown,
        }
    }
}

impl From<Size> for u8 {
    fn from(size: Size) -> u8 {
        match size {
            Size::Any | Size::Unknown => 0,
            Size::Byte => 1,
            Size::Word => 2,
            Size::Dword => 4,
            Size::Qword => 8,
            Size::B80 => 10,
            Size::Xword => 16,
            Size::Yword => 32,
            Size::Zword => 64,
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
            Self::B80 => write!(form, "b80"),
            Self::Word => write!(form, "word"),
            Self::Dword => write!(form, "dword"),
            Self::Qword => write!(form, "qword"),
            Self::Xword => write!(form, "xword"),
            Self::Yword => write!(form, "yword"),
            Self::Any => write!(form, "{{any}}"),
            Self::Unknown => write!(form, "{{unknown}}"),
            Self::Zword => write!(form, "zword"),
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
