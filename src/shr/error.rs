// pasm - src/shr/error.rs
// -----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::color::{ColString, Color, Modifier};
use crate::shr::booltable::BoolTable8;
use crate::{RString, CLI};
use std::{
    fmt::{Display, Error, Formatter},
    fs::{File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    sync::LazyLock,
};

static ERR_CTX: LazyLock<(File, PathBuf)> = LazyLock::new(|| {
    let path = PathBuf::from(CLI.get_kv_arg("-i").unwrap());
    (
        OpenOptions::new()
            .write(false)
            .append(false)
            .truncate(false)
            .create_new(false)
            .create(false)
            .read(true)
            .open(&path)
            .unwrap(),
        path,
    )
});

static FILE: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut buf = String::new();
    (&ERR_CTX.0).read_to_string(&mut buf).unwrap();
    buf.lines().map(|s| s.to_string()).collect::<Vec<String>>()
});

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum ExceptionType {
    Warn,
    Error,
    Info,
}

pub const HAS_LINE: u8 = 0x0;
pub const HAS_MSG: u8 = 0x1;
pub const HAS_TIP: u8 = 0x2;

#[derive(Debug, Clone, Copy, PartialEq)]
struct RASMErrorInfo {
    extype: ExceptionType,
    guardians: BoolTable8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RASMError {
    line: usize,
    msg: RString,
    tip: RString,
    etype: RASMErrorInfo,
}

pub fn print_error(r: RASMError, file_name: &Path) {
    let mut fileb = String::new();
    File::read_to_string(&mut File::open(file_name).unwrap(), &mut fileb).unwrap();
    let file: Vec<String> = fileb.lines().map(|s| s.to_string()).collect();

    let ctx = if let Some(line) = r.get_line() {
        Some(&file[line])
    } else {
        None
    };

    println!(
        "{}:\n\tin {}{}{}{}{}",
        r.etype.extype,
        ColString::new(file_name.to_string_lossy()).set_color(Color::YELLOW),
        if let Some(line) = r.get_line() {
            format!(
                " at line {}",
                ColString::new(line + 1).set_color(Color::YELLOW)
            )
        } else {
            "".to_string()
        },
        if let Some(ctx) = ctx {
            format!(
                "\n\t{}",
                ColString::new(ctx.trim())
                    .set_color(Color::GREEN)
                    .set_modf(Modifier::Bold)
            )
        } else {
            "".to_string()
        },
        if let Some(msg) = r.get_msg() {
            format!("\n\t---\n\t{}", msg)
        } else {
            "".to_string()
        },
        if let Some(tip) = r.get_tip() {
            format!(
                "\n\t{} {}",
                ColString::new("tip:")
                    .set_color(Color::BLUE)
                    .set_modf(Modifier::Bold),
                ColString::new(tip)
                    .set_color(Color::BLUE)
                    .set_modf(Modifier::Bold)
            )
        } else {
            "".to_string()
        }
    );
}

impl Display for RASMError {
    fn fmt(&self, frm: &mut Formatter<'_>) -> Result<(), Error> {
        let ctx = if let Some(line) = self.get_line() {
            Some(&FILE[line])
        } else {
            None
        };

        writeln!(
            frm,
            "{}:\n\tin {}{}{}{}{}",
            self.etype.extype,
            ColString::new(ERR_CTX.1.to_string_lossy()).set_color(Color::YELLOW),
            if let Some(line) = self.get_line() {
                format!(
                    " at line {}",
                    ColString::new(line + 1).set_color(Color::YELLOW)
                )
            } else {
                "".to_string()
            },
            if let Some(ctx) = ctx {
                format!(
                    "\n\t{}",
                    ColString::new(ctx.trim())
                        .set_color(Color::GREEN)
                        .set_modf(Modifier::Bold)
                )
            } else {
                "".to_string()
            },
            if let Some(msg) = self.get_msg() {
                format!("\n\t---\n\t{}", msg)
            } else {
                "".to_string()
            },
            if let Some(tip) = self.get_tip() {
                format!(
                    "\n\t{} {}",
                    ColString::new("tip:")
                        .set_color(Color::BLUE)
                        .set_modf(Modifier::Bold),
                    ColString::new(tip)
                        .set_color(Color::BLUE)
                        .set_modf(Modifier::Bold)
                )
            } else {
                "".to_string()
            }
        )
    }
}

impl Display for ExceptionType {
    fn fmt(&self, frm: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            frm,
            "{}",
            if let Self::Warn = self {
                ColString::new("warn").set_color(Color::YELLOW)
            } else if let Self::Error = self {
                ColString::new("error").set_color(Color::RED)
            } else {
                ColString::new("info").set_color(Color::BLUE)
            }
        )
    }
}

impl RASMError {
    fn get_tip(&self) -> Option<RString> {
        if self.etype.guardians.get(HAS_TIP).unwrap_or_default() {
            Some(self.tip.clone())
        } else {
            None
        }
    }
    fn get_msg(&self) -> Option<RString> {
        if self.etype.guardians.get(HAS_MSG).unwrap_or_default() {
            Some(self.msg.clone())
        } else {
            None
        }
    }
    pub fn msg(msg: impl ToString) -> Self {
        Self {
            line: 0,
            msg: msg.to_string().into(),
            tip: RString::from(""),
            etype: RASMErrorInfo {
                extype: ExceptionType::Error,
                guardians: BoolTable8::new().setc(HAS_MSG, true),
            },
        }
    }
    pub fn no_tip(line: Option<usize>, msg: Option<impl ToString>) -> Self {
        Self {
            line: line.unwrap_or(0),
            msg: if let Some(ref m) = msg {
                m.to_string().into()
            } else {
                RString::from("")
            },
            tip: RString::from(""),
            etype: RASMErrorInfo {
                extype: ExceptionType::Error,
                guardians: BoolTable8::new()
                    .setc(HAS_LINE, line.is_some())
                    .setc(HAS_MSG, msg.is_some()),
            },
        }
    }
    pub fn with_tip(
        line: Option<usize>,
        msg: Option<impl ToString>,
        tip: Option<impl ToString>,
    ) -> Self {
        Self {
            line: line.unwrap_or(0),
            msg: if let Some(ref m) = msg {
                m.to_string().into()
            } else {
                RString::from("")
            },
            tip: if let Some(ref m) = tip {
                m.to_string().into()
            } else {
                RString::from("")
            },
            etype: RASMErrorInfo {
                extype: ExceptionType::Error,
                guardians: BoolTable8::new()
                    .setc(HAS_LINE, line.is_some())
                    .setc(HAS_MSG, msg.is_some())
                    .setc(HAS_TIP, tip.is_some()),
            },
        }
    }
    pub fn get_line(&self) -> Option<usize> {
        if self.etype.guardians.get(HAS_LINE).unwrap_or_default() {
            Some(self.line)
        } else {
            None
        }
    }
    pub fn set_line(&mut self, newline: usize) {
        self.line = newline;
        self.etype.guardians.set(HAS_LINE, true);
    }
    pub fn warn(msg: String) {
        let err = Self {
            line: 0,
            msg: msg.to_string().into(),
            tip: RString::from(""),
            etype: RASMErrorInfo {
                extype: ExceptionType::Error,
                guardians: BoolTable8::new().setc(HAS_MSG, true),
            },
        };
        println!("{}", err);
    }
}

pub struct Blank;

#[allow(clippy::to_string_trait_impl)]
impl ToString for Blank {
    fn to_string(&self) -> String {
        String::new()
    }
}
