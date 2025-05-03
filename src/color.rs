// rasmx86_64 - src/color.rs
// -------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::CLI;

use std::fmt::{Display, Error as FmtError, Formatter};

#[derive(Debug, Clone, Copy)]
pub enum BaseColor {
    BLACK = 0,
    RED = 1,
    GREEN = 2,
    YELLOW = 3,
    BLUE = 4,
    PURPLE = 5,
    CYAN = 6,
    WHITE = 7,
}

#[derive(Debug, Clone, Copy)]
pub enum Modifier {
    Regular = 0,
    Bold = 1,
    Underline = 4,
}

#[derive(Debug, Clone)]
pub struct ColString {
    val: String,
    col: BaseColor,
    mdf: Modifier,
}

impl ColString {
    pub fn new(arg: impl ToString) -> Self {
        Self {
            val: arg.to_string(),
            col: BaseColor::WHITE,
            mdf: Modifier::Regular,
        }
    }
    pub fn set_color(self, col: BaseColor) -> Self {
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
        if CLI.nocolor {
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
