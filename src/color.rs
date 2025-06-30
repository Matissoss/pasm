// pasm - src/color.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use crate::cli::*;

use std::fmt::{Display, Error as FmtError, Formatter};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Color {
    BLACK = 0,
    RED = 1,
    GREEN = 2,
    YELLOW = 3,
    BLUE = 4,
    PURPLE = 5,
    CYAN = 6,
    WHITE = 7,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Modifier {
    Regular = 0,
    Bold = 1,
    Underline = 4,
}

#[derive(Debug, Clone)]
pub struct ColString {
    val: String,
    col: Color,
    mdf: Modifier,
}

impl ColString {
    pub fn new(arg: impl ToString) -> Self {
        Self {
            val: arg.to_string(),
            col: Color::WHITE,
            mdf: Modifier::Regular,
        }
    }
    pub fn set_color(self, col: Color) -> Self {
        Self {
            val: self.val,
            col,
            mdf: self.mdf,
        }
    }
    pub fn set_modf(self, mdf: Modifier) -> Self {
        Self {
            val: self.val,
            col: self.col,
            mdf,
        }
    }
}

impl Display for ColString {
    fn fmt(&self, frm: &mut Formatter<'_>) -> Result<(), FmtError> {
        if cli_nocolor(&CLI) {
            write!(frm, "{}", self.val)
        } else {
            write!(
                frm,
                "\x1b[{};{}m{}\x1b[0m",
                self.mdf as u8,
                self.col as u8 + 30,
                self.val
            )
        }
    }
}
