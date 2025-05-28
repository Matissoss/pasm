// rasmx86_64 - src/shr/error.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::color::{ColString, Color, Modifier};
use crate::CLI;
use std::{
    fmt::{Display, Error, Formatter},
    fs::{File, OpenOptions},
    io::Read,
    path::PathBuf,
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

#[derive(Debug, Clone, PartialEq)]
pub struct RASMError {
    line: Option<usize>,
    etype: ExceptionType,
    msg: Option<String>,
    tip: Option<String>,
}

impl Display for RASMError {
    fn fmt(&self, frm: &mut Formatter<'_>) -> Result<(), Error> {
        let line = if let Some(line) = self.line {
            Some(line - 1)
        } else {
            None
        };

        let ctx = if let Some(line) = line {
            Some(&FILE[line])
        } else {
            None
        };

        writeln!(
            frm,
            "{}:\n\tin {}{}{}{}{}",
            self.etype,
            ColString::new(ERR_CTX.1.to_string_lossy()).set_color(Color::YELLOW),
            if let Some(line) = line {
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
            if let Some(msg) = &self.msg {
                format!("\n\t---\n\t{}", msg)
            } else {
                "".to_string()
            },
            if let Some(tip) = &self.tip {
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
    pub fn no_tip(line: Option<usize>, msg: Option<impl ToString>) -> Self {
        Self {
            line,
            etype: ExceptionType::Error,
            msg: if let Some(m) = msg {
                Some(m.to_string())
            } else {
                None
            },
            tip: None,
        }
    }
    pub fn with_tip(
        line: Option<usize>,
        msg: Option<impl ToString>,
        tip: Option<impl ToString>,
    ) -> Self {
        Self {
            line,
            etype: ExceptionType::Error,
            msg: if let Some(m) = msg {
                Some(m.to_string())
            } else {
                None
            },
            tip: if let Some(t) = tip {
                Some(t.to_string())
            } else {
                None
            },
        }
    }
    pub fn get_line(&self) -> Option<&usize> {
        self.line.as_ref()
    }
    pub fn set_line(&mut self, newline: usize) {
        self.line = Some(newline);
    }
    pub fn warn(msg: String) {
        let err = Self {
            line: None,
            etype: ExceptionType::Warn,
            msg: Some(msg),
            tip: None,
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
