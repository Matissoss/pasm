// pasm - src/shr/section.rs
// -------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{booltable, label, visibility::Visibility};

const GLOBAL: u8 = 0x1;
const ALLOC_FLAG: u8 = 0x2;
const WRITE_FLAG: u8 = 0x3;
const EXEC_FLAG: u8 = 0x4;

#[derive(PartialEq, Clone, Debug, Default)]
pub struct SlimSection<'a> {
    pub name: &'a str,
    pub size: usize,
    pub offset: usize,
    pub align: u16,
    pub attributes: SectionAttributes,
    pub bits: u8,
}

#[derive(PartialEq, Clone, Debug, Default)]
pub struct Section<'a> {
    pub name: &'a str,
    pub content: Vec<label::Label<'a>>,
    pub size: usize,
    pub offset: usize,
    pub align: u16,
    pub attributes: SectionAttributes,
    pub bits: u8,
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
#[repr(transparent)]
pub struct SectionAttributes {
    flags: booltable::BoolTable8,
}

impl SectionAttributes {
    pub const fn new() -> Self {
        Self {
            flags: booltable::BoolTable8::new(),
        }
    }
    pub const fn set_global(&mut self, b: bool) {
        self.flags.set(GLOBAL, b);
    }
    pub const fn set_alloc(&mut self, b: bool) {
        self.flags.set(ALLOC_FLAG, b);
    }
    pub const fn set_exec(&mut self, b: bool) {
        self.flags.set(EXEC_FLAG, b);
    }
    pub const fn set_write(&mut self, b: bool) {
        self.flags.set(WRITE_FLAG, b);
    }
    pub fn write(&self) -> bool {
        self.flags.get(WRITE_FLAG).unwrap_or(false)
    }
    pub fn exec(&self) -> bool {
        self.flags.get(EXEC_FLAG).unwrap_or(false)
    }
    pub fn alloc(&self) -> bool {
        self.flags.get(ALLOC_FLAG).unwrap_or(false)
    }
    pub fn visibility(&self) -> Visibility {
        if self.flags.get(GLOBAL).unwrap_or(false) {
            Visibility::Public
        } else {
            Visibility::Local
        }
    }
}
