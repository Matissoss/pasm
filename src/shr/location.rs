// pasm - src/shr/location.rs
// --------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::RString;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Location {
    pub line: usize, // if 0, then NULL
    pub file: RString,
}

impl Location {
    pub fn get_file(&self) -> Option<RString> {
        if self.file.is_empty() {
            None
        } else {
            Some(self.file.clone())
        }
    }
    pub const fn get_char(&self) -> Option<usize> {
        None
    }
    pub const fn get_line(&self) -> Option<usize> {
        if self.line == 0 {
            None
        } else {
            Some(self.line)
        }
    }
    pub const fn set_char(&mut self, _char: usize) {
        //self.char = char;
    }
    pub const fn set_line(&mut self, line: usize) {
        self.line = line;
    }
    pub fn set_file(&mut self, path: RString) {
        self.file = path.clone();
    }
}
