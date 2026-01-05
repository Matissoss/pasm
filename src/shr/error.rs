// pasm - src/shr/error.rs
// -----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::cli::CLI;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Error {
    line: u64,
    msg: Box<str>,
    error_code: u64,
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
        writeln!(f, "error[a{:04}]: {}", self.line, self.msg)?;
        if self.line != 0 {
            writeln!(f, "---> at line {}", self.line)?;
            if let Some(pth) = CLI.infile() {
                // pls don't cancel me for this horrible code, i'll try to optimize this better
                // later if Rust allows me to use global scope variables, i promise
                let file_content: Vec<String> = std::fs::read_to_string(pth)
                    .unwrap()
                    .lines()
                    .map(|s| s.to_string())
                    .collect();
                for i in (self.line - 1)..=(self.line + 1) {
                    if let Some(l) = file_content.get((i as usize) - 1) {
                        if i == self.line {
                            writeln!(f, "\t->| {l}")?;
                        } else {
                            writeln!(f, "\t  | {l}")?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl PartialEq for Error {
    fn eq(&self, rhs: &Self) -> bool {
        self.error_code == rhs.error_code
    }
}

impl Error {
    pub fn new_wline(msg: impl ToString, ecd: u64, line: usize) -> Self {
        let mut s = Self::new(msg, ecd);
        s.set_line(line);
        s
    }
    pub fn new(msg: impl ToString, ecd: u64) -> Self {
        Self {
            line: 0,
            msg: msg.to_string().into(),
            error_code: ecd,
        }
    }
    pub fn msg(&self) -> &str {
        &self.msg
    }
    pub fn set_line(&mut self, line: usize) {
        self.line = line as u64;
    }
    pub fn get_line(&self) -> usize {
        self.line as usize
    }
}
