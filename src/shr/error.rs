// pasm - src/shr/error.rs
// -----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::RString;
use std::{fmt::Display, path::PathBuf};

use std::mem::MaybeUninit;

// new error type
#[derive(Debug)]
pub struct RError {
    // file path
    fl: MaybeUninit<PathBuf>,
    // file line - if 0 then not set
    ln: usize,

    // additional context
    actx: usize,
    // message
    msg: RString,
    // layout:
    //  0bXX_Y[Y]..
    //  - X0: reserved
    //  - X1: if 1 then fl is some
    //  - Y[Y]: error code
    meta: u16,
}

impl Clone for RError {
    fn clone(&self) -> Self {
        let mut s = Self::new(self.msg.clone(), self.meta);
        if let Some(f) = self.get_file() {
            s.set_file(f.clone());
        }
        if let Some(f) = self.get_line() {
            s.set_line(f);
        }
        s
    }
}

impl Display for RError {
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
        f.write_str("\n")?;
        f.write_str(&ColString::new("error").set_color(Color::RED).to_string())?;
        f.write_str(&format!("[{:03}]: ", self.meta & 0x3FFF))?;
        f.write_str(&self.msg)?;
        f.write_str("\n")?;
        if let Some(file) = self.get_file() {
            if let Some(ln) = self.get_line() {
                f.write_str(&" ".repeat(ln.to_string().len() + 1))?;
                f.write_str("--> ")?;
            }
            f.write_str(&format!("{}", file.to_string_lossy()))?;
            if let Some(ln) = self.get_line() {
                f.write_str(&format!(":{ln}"))?;
            }
            f.write_str("\n")?;
            let content = std::fs::read_to_string(file).expect("Internal Error: couldnt read file");
            let content: Vec<&str> = content.lines().collect();

            let linepad = if let (Some(ln), Some(actx)) = (self.get_line(), self.get_actx()) {
                ln.to_string().len().max(actx.to_string().len()) + 2
            } else if let Some(ln) = self.get_line() {
                ln.to_string().len() + 2
            } else {
                0
            };

            if let Some(mut ln) = self.get_line() {
                ln -= 1;
                if ln != 0 {
                    if let Some(line) = content.get(ln - 1) {
                        f.write_str(&format!("{}| {}", " ".repeat(linepad), line))?;
                    } else {
                        f.write_str(&format!("{}|", " ".repeat(linepad)))?;
                    }
                } else {
                    f.write_str(&format!("{}|", " ".repeat(linepad)))?;
                }
                if let Some(line) = content.get(ln) {
                    f.write_str(&format!("\n{} >| {}", ln + 1, line))?;
                }
                if let Some(line) = content.get(ln + 1) {
                    f.write_str(&format!("\n{}| {}", " ".repeat(linepad), line))?;
                } else {
                    f.write_str(&format!("\n{}|", " ".repeat(linepad)))?;
                }
            }
            if let Some(mut ln) = self.get_actx() {
                f.write_str("\nadditional context:\n")?;
                ln -= 1;
                if ln != 0 {
                    if let Some(line) = content.get(ln - 1) {
                        f.write_str(&format!("{}| {}", " ".repeat(linepad), line))?;
                    } else {
                        f.write_str(&format!("{}|", " ".repeat(linepad)))?;
                    }
                } else {
                    f.write_str(&format!("{}|", " ".repeat(linepad)))?;
                }
                if let Some(line) = content.get(ln) {
                    f.write_str(&format!("\n{} >| {}", ln + 1, line))?;
                }
                if let Some(line) = content.get(ln + 1) {
                    f.write_str(&format!("\n{}| {}", " ".repeat(linepad), line))?;
                } else {
                    f.write_str(&format!("\n{}|", " ".repeat(linepad)))?;
                }
            }
        }
        f.write_str(&format!(
            "\n{}: go to `pasm/docs/error-spec.md#e{:03}` for more information.",
            ColString::new("help").set_color(Color::GREEN),
            self.meta & 0x3FFF
        ))?;
        f.write_str("\n")?;

        Ok(())
    }
}

impl PartialEq for RError {
    fn eq(&self, rhs: &Self) -> bool {
        self.meta & 0x3FFF == rhs.meta & 0x3FFF
    }
}

impl RError {
    pub fn new_wline_actx(msg: impl ToString, ecd: u16, line: usize, atcx: usize) -> Self {
        let mut s = Self::new(msg, ecd);
        s.set_line(line);
        s.actx = atcx;
        s
    }
    pub fn new_wline(msg: impl ToString, ecd: u16, line: usize) -> Self {
        let mut s = Self::new(msg, ecd);
        s.set_line(line);
        s
    }
    pub fn new(msg: impl ToString, ecd: u16) -> Self {
        Self {
            fl: MaybeUninit::uninit(),
            ln: 0,
            msg: msg.to_string().into(),
            meta: ecd & 0x3FFF,
            actx: 0,
        }
    }
    pub fn msg(&self) -> &RString {
        &self.msg
    }
    pub fn set_line(&mut self, line: usize) {
        self.ln = line;
    }
    pub fn set_file(&mut self, fl: PathBuf) {
        self.fl = MaybeUninit::new(fl);
        self.meta |= 1 << 14;
    }
    pub fn get_file(&self) -> Option<&PathBuf> {
        if self.meta & 1 << 14 == 1 << 14 {
            Some(unsafe { self.fl.assume_init_ref() })
        } else {
            None
        }
    }
    pub fn get_actx(&self) -> Option<usize> {
        if self.actx == 0 {
            None
        } else {
            Some(self.actx)
        }
    }
    pub fn get_line(&self) -> Option<usize> {
        if self.ln == 0 {
            None
        } else {
            Some(self.ln)
        }
    }
}
