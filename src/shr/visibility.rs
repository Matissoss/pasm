// pasm - src/shr/visibility.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

#[derive(Default, PartialEq, Clone, Copy)]
pub enum Visibility {
    #[default]
    Local,
    Public,
    Weak,
    Anonymous,
    Protected,
    Extern,
}

impl Visibility {
    // returns 3 bits
    pub const fn se(&self) -> u8 {
        match self {
            Self::Local => 0b000,
            Self::Public => 0b001,
            Self::Weak => 0b010,
            Self::Anonymous => 0b011,
            Self::Protected => 0b100,
            Self::Extern => 0b101,
        }
    }
    pub const fn de(v: u8) -> Self {
        match v {
            0b000 => Self::Local,
            0b001 => Self::Public,
            0b010 => Self::Weak,
            0b011 => Self::Anonymous,
            0b100 => Self::Protected,
            0b101 => Self::Extern,
            _ => Self::Local,
        }
    }
}
