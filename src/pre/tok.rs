// pasm - src/pre/tok.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

/// STATS:
///     - Size: 6B
///     - Align: 2B
///     - Padding: 1B
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Token {
    ClosureOpen,    // (
    ClosureClose,   // )
    Semicolon,      // ;
    Colon,          // :
    Comma,          // ,
    Dot,            // .
    StringDelimeter,// "
    CharDelimeter,  // '
    String(u16, u16),
}

// TODO: 
//  replace with SmallVec aka Vector that can be allocated on both stack and/or heap if fixed
//  size is too small
/// DESC:
/// Splits line into small tokens that later can be used for merging into more complex tokens.
pub fn tokl<'a>(line: &'a str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut slice_start: u16 = 0;
    let mut slice_end: u16 = 0;
    for b in line.as_bytes() {
        match *b {
            b'\'' => {
                if slice_start != slice_end {
                    tokens.push(Token::String(slice_start, slice_end));
                }
                slice_end += 1;
                slice_start = slice_end;
                tokens.push(Token::CharDelimeter);
            },
            b'"' => {
                if slice_start != slice_end {
                    tokens.push(Token::String(slice_start, slice_end));
                }
                slice_end += 1;
                slice_start = slice_end;
                tokens.push(Token::StringDelimeter);
            },
            b'(' => {
                if slice_start != slice_end {
                    tokens.push(Token::String(slice_start, slice_end));
                }
                slice_end += 1;
                slice_start = slice_end;
                tokens.push(Token::ClosureOpen);
            },
            b')' => {
                if slice_start != slice_end {
                    tokens.push(Token::String(slice_start, slice_end));
                }
                slice_end += 1;
                slice_start = slice_end;
                tokens.push(Token::ClosureClose);
            },
            b'.' => {
                if slice_start != slice_end {
                    tokens.push(Token::String(slice_start, slice_end));
                }
                slice_end += 1;
                slice_start = slice_end;
                tokens.push(Token::Dot);
            },
            b',' => {
                if slice_start != slice_end {
                    tokens.push(Token::String(slice_start, slice_end));
                }
                slice_end += 1;
                slice_start = slice_end;
                tokens.push(Token::Comma);
            },
            b':' => {
                if slice_start != slice_end {
                    tokens.push(Token::String(slice_start, slice_end));
                }
                slice_end += 1;
                slice_start = slice_end;
                tokens.push(Token::Colon);
            }
            b';' => {
                if slice_start != slice_end {
                    tokens.push(Token::String(slice_start, slice_end));
                }
                slice_end += 1;
                slice_start = slice_end;
                tokens.push(Token::Semicolon);
            }
            b' '|b'\t' => {
                if slice_start != slice_end {
                    tokens.push(Token::String(slice_start, slice_end));
                }
                slice_end += 1;
                slice_start = slice_end;
            }
            _ => {
                slice_end += 1;
            },
        }
    }
    return tokens;
}

#[cfg(test)]
mod tok_test {
    use super::*;
    #[test]
    pub fn tokl_test() {
        use Token::*;
        assert_eq!(
            tokl("\"String\", ('closure') : ; free string ."),
            vec![
                StringDelimeter,
                String(1, 7),
                StringDelimeter,
                Comma,
                ClosureOpen,
                CharDelimeter,
                String(12, 19),
                CharDelimeter,
                ClosureClose,
                Colon,
                Semicolon,
                String(26, 30),
                String(31, 37),
                Dot,
            ]
        );
    }
}
