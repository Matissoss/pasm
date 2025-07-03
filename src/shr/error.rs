// pasm - src/shr/error.rs
// -----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::location::Location;
use crate::RString;

use std::{fmt::Display, path::PathBuf};

// new error type
#[derive(Debug, Clone)]
pub struct Error {
    // file path
    location: Location,
    context: Box<Location>,

    // message
    msg: RString,

    error_code: u16,
}

impl Display for Error {
    // error format (rust-like error format):
    //
    // {EXCEPTION_TYPE}[{ERROR_CODE}]: {MSG}
    //   {FILE:LINE}
    //   | {LINE} - 1
    // L>| {LINE} - 0
    //   | {LINE} + 1
    // help: go to `{SOURCE_CODE_REPO}/docs/error-spec.md#e[{ERROR_CODE}]` for more info
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use crate::color::*;
        writeln!(
            f,
            "{}[{:03}]: {}",
            ColString::new("error").set_color(Color::RED),
            self.error_code,
            self.msg,
        )?;

        if let Some(file) = self.get_file() {
            write!(f, "--> in {file}")?;
            if let Some(line) = self.get_line() {
                writeln!(
                    f,
                    " at {line}{}",
                    if let Some(c) = self.location.get_char() {
                        format!(":{c}")
                    } else {
                        "".to_string()
                    }
                )?;
            }

            let file = std::fs::read_to_string(&*file).expect("should exist");
            let src_file = file.lines().collect::<Vec<&str>>();

            if let Some(line_num) = self.get_line() {
                let line_padding = line_num.to_string().len() + 2;

                let start = if line_num == 1 { 0 } else { line_num - 2 };
                let destination = line_num;
                let mut context = 0;

                while context < 2 {
                    if let Some(l) = src_file.get(start + context) {
                        if start + context == destination - 1 {
                            writeln!(f, "{} >| {}", destination, l)?;
                        } else {
                            writeln!(f, "{}| {}", " ".repeat(line_padding), l)?;
                        }
                    }
                    context += 1;
                }

                let uctx = &*self.context;
                if let Some(ctx_file) = uctx.get_file() {
                    let ctx_file = std::fs::read_to_string(&*ctx_file).expect("should exist");
                    let ctx_file = ctx_file.lines().collect::<Vec<&str>>();
                    if let Some(ctx_line) = uctx.get_line() {
                        write!(f, "\nadditional context:\n")?;
                        let line_padding = ctx_line.to_string().len() + 2;
                        let start = uctx.line - 2;
                        let destination = uctx.line;
                        let mut context = 0;

                        while context <= 3 {
                            if let Some(l) = ctx_file.get(start + context) {
                                if start + context == destination - 1 {
                                    writeln!(f, "{} >| {}", destination, l)?;
                                } else {
                                    writeln!(f, "{}| {}", " ".repeat(line_padding), l)?;
                                }
                            }
                            context += 1;
                        }
                    }
                }
            }
        }

        write!(
            f,
            "\n{}: go to `pasm/docs/error-spec.md#e{:03}` for more information.\n",
            ColString::new("help").set_color(Color::GREEN),
            self.error_code
        )?;

        Ok(())
    }
}

impl PartialEq for Error {
    fn eq(&self, rhs: &Self) -> bool {
        self.error_code == rhs.error_code
    }
}

impl Error {
    pub fn new_wline_actx(
        msg: impl ToString,
        ecd: u16,
        line: usize,
        actx: usize,
        file: RString,
    ) -> Self {
        let mut s = Self::new(msg, ecd);
        s.set_line(line);
        s.context.file = file.clone();
        s.location.file = file;
        s.context.line = actx;
        s
    }
    pub fn new_wline(msg: impl ToString, ecd: u16, line: usize) -> Self {
        let mut s = Self::new(msg, ecd);
        s.set_line(line);
        s
    }
    pub fn new(msg: impl ToString, ecd: u16) -> Self {
        Self {
            context: Box::new(Location::default()),
            location: Location::default(),
            msg: msg.to_string().into(),
            error_code: ecd,
        }
    }
    pub fn msg(&self) -> &RString {
        &self.msg
    }
    pub fn set_line(&mut self, line: usize) {
        self.location.set_line(line);
    }
    pub fn set_file(&mut self, fl: PathBuf) {
        self.location
            .set_file(fl.to_string_lossy().to_string().into());
    }
    pub fn set_context(&mut self, ctx: Location) {
        self.context = Box::new(ctx);
    }
    pub fn get_file(&self) -> Option<RString> {
        self.location.get_file()
    }
    pub fn get_line(&self) -> Option<usize> {
        self.location.get_line()
    }
}

#[cfg(test)]
mod t {
    #[test]
    fn t() {
        use super::Error;
        let er = Error::new("h", 1);
        assert_eq!(er.get_file(), None);
        assert_eq!(er.get_line(), None);
    }
}
