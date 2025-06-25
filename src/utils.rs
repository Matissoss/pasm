// pasm - src/utils.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use std::time::SystemTime;
#[inline(always)]
pub fn vtimed_print(str: &'static str, tm: SystemTime) {
    println!(
        "{str} took {:03.16}s",
        SystemTime::now()
            .duration_since(tm)
            .unwrap_or_default()
            .as_secs_f32(),
    );
}

pub fn split_str_owned(s: &str, chr: char) -> Vec<String> {
    let mut tmp_buf = Vec::new();
    let mut strs = Vec::new();

    let chrb = chr as u8;
    for b in s.as_bytes() {
        if b == &chrb {
            strs.push(String::from_utf8_lossy(&tmp_buf).to_string());
            tmp_buf.clear();
        } else {
            tmp_buf.push(*b);
        }
    }
    if !tmp_buf.is_empty() {
        strs.push(String::from_utf8_lossy(&tmp_buf).to_string());
    }
    strs
}
