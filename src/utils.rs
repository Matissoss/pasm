// pasm - src/utils.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use std::time::SystemTime;
#[inline(always)]
pub fn vtimed_print(str: &str, tm: SystemTime) {
    println!(
        "{str} took {:03.16}s",
        SystemTime::now()
            .duration_since(tm)
            .unwrap_or_default()
            .as_secs_f32(),
    );
}

pub struct LineIter<'a> {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub slice: &'a [u8],
}

impl<'a> LineIter<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        Self {
            start: 0,
            end: 0,
            line: 0,
            slice,
        }
    }
    pub fn next(&mut self) -> Option<(usize, &'a str)> {
        if self.end >= self.slice.len() {
            None
        } else {
            for b in self.start..self.slice.len() {
                if self.slice[b] == b'\n' {
                    let line_s = std::str::from_utf8(&self.slice[self.start..self.end])
                        .expect("pasm source code needs to be encoded in UTF-8");
                    self.end += 1;
                    self.start = self.end;
                    self.line += 1;
                    return Some((self.line - 1, line_s));
                } else {
                    self.end += 1;
                }
            }
            if self.end != self.start {
                self.end = self.slice.len();
                let line_s = Some((
                    self.line,
                    std::str::from_utf8(&self.slice[self.start..self.end])
                        .expect("pasm source code needs to be encoded in UTF-8"),
                ));
                self.start = self.end;
                self.line += 1;
                line_s
            } else {
                None
            }
        }
    }
}

pub fn andn<T>(lhs: T, rhs: T) -> T
where
    T: std::ops::Not<Output = T> + std::ops::BitAnd<Output = T>,
{
    !lhs & rhs
}

pub unsafe fn cstring<'a>(s: *const u8) -> &'a str {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    std::str::from_utf8_unchecked(std::slice::from_raw_parts(s, len))
}

/// splits line more intelligently (so mov rax, ',' will work)          
pub fn split_once_intelligent(line: &str, c: char) -> Option<(&str, &str)> {
    let mut str_closure = false;
    for (i, b) in line.as_bytes().iter().enumerate() {
        if b == &b'"' || b == &b'\'' {
            str_closure = !str_closure;
        } else if b == &(c as u8) && !str_closure {
            return Some((&line[0..i], &line[i + 1..]));
        }
    }
    None
}

pub fn split_str_ref(s: &[u8], chr: char) -> Vec<&str> {
    let mut start = 0;
    let mut end = 0;

    let mut strs = Vec::new();

    let chrb = chr as u8;
    for b in s {
        if b == &chrb {
            if start != end {
                strs.push(
                    std::str::from_utf8(&s[start..end])
                        .expect("pasm source code should be encoded in valid UTF-8"),
                );
            }
            end += 1;
            start = end;
        } else {
            end += 1;
        }
    }
    if start != end {
        strs.push(
            std::str::from_utf8(&s[start..end])
                .expect("pasm source code should be encoded in valid UTF-8"),
        );
    }
    strs
}
