// rasmx86_64 - src/shr/segment.rs
// -------------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    atype::{AType, ToAType},
    mem::Mem,
    reg::Register,
    size::Size,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub segment: Register,
    pub address: Mem,
}

impl ToAType for Segment {
    fn atype(&self) -> AType {
        AType::Memory(self.address.size().unwrap_or(Size::Unknown))
    }
}
