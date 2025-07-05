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
