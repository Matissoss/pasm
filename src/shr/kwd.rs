// pasm - src/shr/kwd.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Keyword {
    Word,
    Byte,
    Bits,
    Any,
    Qword,
    Dword,
    Xword,
    Yword,

    // AVX-512
    Zword,
    Bcst,

    Extern,
    Include,

    // sections
    Section,
    Align,

    Executable,
    Writeable,
    Alloc,

    // symbol referencing
    Deref,
    Ref,

    Rel32,
    Abs32,
    Abs64,

    Rel16,
    Rel8,

    Define,

    // label visibility
    Local,
    Protected,
    Weak,
    Public,
    Anonymous,
    // label type
    Function,
    Object,

    Format,
    Output,
}

impl std::str::FromStr for Keyword {
    type Err = ();
    fn from_str(kwd_str: &str) -> Result<Self, <Self as std::str::FromStr>::Err> {
        if let Some(k) = kwd_fromstr(kwd_str) {
            Ok(k)
        } else {
            Err(())
        }
    }
}

#[inline(always)]
fn s<T>(t: T) -> Option<T> {
    Some(t)
}

const N: Option<Keyword> = None;

pub fn kwd_fromstr(str: &str) -> Option<Keyword> {
    use Keyword::*;
    let r = str.as_bytes();
    match r.len() {
        3 => match r[0] {
            b'a' => match r[1] {
                b'n' => match r[2] {
                    b'y' => s(Any),
                    _ => N,
                },
                _ => N,
            },
            b'r' => match r[1] {
                b'e' => match r[2] {
                    b'f' => s(Ref),
                    _ => N,
                },
                _ => N,
            },
            _ => N,
        },
        4 => match r[0] {
            b'r' => match r[1] {
                b'e' => match r[2] {
                    b'l' => match r[3] {
                        b'8' => s(Rel8),
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'b' => match r[1] {
                b'c' => match r[2] {
                    b's' => match r[3] {
                        b't' => s(Bcst),
                        _ => N,
                    },
                    _ => N,
                },
                b'i' => match r[2] {
                    b't' => match r[3] {
                        b's' => s(Bits),
                        _ => N,
                    },
                    _ => N,
                },
                b'y' => match r[2] {
                    b't' => match r[3] {
                        b'e' => s(Byte),
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'w' => match r[1] {
                b'e' => match r[2] {
                    b'a' => match r[3] {
                        b'k' => s(Weak),
                        _ => N,
                    },
                    _ => N,
                },
                b'o' => match r[2] {
                    b'r' => match r[3] {
                        b'd' => s(Word),
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            _ => N,
        },
        5 => match r[0] {
            b'l' => match r[1] {
                b'o' => match r[2] {
                    b'c' => match r[3] {
                        b'a' => match r[4] {
                            b'l' => s(Local),
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'q' => match r[1] {
                b'w' => match r[2] {
                    b'o' => match r[3] {
                        b'r' => match r[4] {
                            b'd' => s(Qword),
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'r' => match r[1] {
                b'e' => match r[2] {
                    b'l' => match r[3] {
                        b'1' => match r[4] {
                            b'6' => s(Rel16),
                            _ => N,
                        },
                        b'3' => match r[4] {
                            b'2' => s(Rel32),
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'x' => match r[1] {
                b'w' => match r[2] {
                    b'o' => match r[3] {
                        b'r' => match r[4] {
                            b'd' => s(Xword),
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'y' => match r[1] {
                b'w' => match r[2] {
                    b'o' => match r[3] {
                        b'r' => match r[4] {
                            b'd' => s(Yword),
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'z' => match r[1] {
                b'w' => match r[2] {
                    b'o' => match r[3] {
                        b'r' => match r[4] {
                            b'd' => s(Zword),
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'a' => match r[1] {
                b'b' => match r[2] {
                    b's' => match r[3] {
                        b'3' => match r[4] {
                            b'2' => s(Abs32),
                            _ => N,
                        },
                        b'6' => match r[4] {
                            b'4' => s(Abs64),
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                b'l' => match r[2] {
                    b'i' => match r[3] {
                        b'g' => match r[4] {
                            b'n' => s(Align),
                            _ => N,
                        },
                        _ => N,
                    },
                    b'l' => match r[3] {
                        b'o' => match r[4] {
                            b'c' => s(Alloc),
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'd' => match r[1] {
                b'e' => match r[2] {
                    b'r' => match r[3] {
                        b'e' => match r[4] {
                            b'f' => s(Deref),
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                b'w' => match r[2] {
                    b'o' => match r[3] {
                        b'r' => match r[4] {
                            b'd' => s(Dword),
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            _ => N,
        },
        6 => match r[0] {
            b'd' => match r[1] {
                b'e' => match r[2] {
                    b'f' => match r[3] {
                        b'i' => match r[4] {
                            b'n' => match r[5] {
                                b'e' => s(Define),
                                _ => N,
                            },
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'e' => match r[1] {
                b'x' => match r[2] {
                    b't' => match r[3] {
                        b'e' => match r[4] {
                            b'r' => match r[5] {
                                b'n' => s(Extern),
                                _ => N,
                            },
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'p' => match r[1] {
                b'u' => match r[2] {
                    b'b' => match r[3] {
                        b'l' => match r[4] {
                            b'i' => match r[5] {
                                b'c' => s(Public),
                                _ => N,
                            },
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'f' => match r[1] {
                b'o' => match r[2] {
                    b'r' => match r[3] {
                        b'm' => match r[4] {
                            b'a' => match r[5] {
                                b't' => s(Format),
                                _ => N,
                            },
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'o' => match r[1] {
                b'b' => match r[2] {
                    b'j' => match r[3] {
                        b'e' => match r[4] {
                            b'c' => match r[5] {
                                b't' => s(Object),
                                _ => N,
                            },
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                b'u' => match r[2] {
                    b't' => match r[3] {
                        b'p' => match r[4] {
                            b'u' => match r[5] {
                                b't' => s(Output),
                                _ => N,
                            },
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            _ => N,
        },
        7 => match r[0] {
            b'i' => match r[1] {
                b'n' => match r[2] {
                    b'c' => match r[3] {
                        b'l' => match r[4] {
                            b'u' => match r[5] {
                                b'd' => match r[6] {
                                    b'e' => s(Include),
                                    _ => N,
                                },
                                _ => N,
                            },
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b's' => match r[1] {
                b'e' => match r[2] {
                    b'c' => match r[3] {
                        b't' => match r[4] {
                            b'i' => match r[5] {
                                b'o' => match r[6] {
                                    b'n' => s(Section),
                                    _ => N,
                                },
                                _ => N,
                            },
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            _ => N,
        },
        8 => match r[0] {
            b'f' => match r[1] {
                b'u' => match r[2] {
                    b'n' => match r[3] {
                        b'c' => match r[4] {
                            b't' => match r[5] {
                                b'i' => match r[6] {
                                    b'o' => match r[7] {
                                        b'n' => s(Function),
                                        _ => N,
                                    },
                                    _ => N,
                                },
                                _ => N,
                            },
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            _ => N,
        },
        9 => match r[0] {
            b'w' => match r[1] {
                b'r' => match r[2] {
                    b'i' => match r[3] {
                        b't' => match r[4] {
                            b'e' => match r[5] {
                                b'a' => match r[6] {
                                    b'b' => match r[7] {
                                        b'l' => match r[8] {
                                            b'e' => s(Writeable),
                                            _ => N,
                                        },
                                        _ => N,
                                    },
                                    _ => N,
                                },
                                _ => N,
                            },
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'a' => match r[1] {
                b'n' => match r[2] {
                    b'o' => match r[3] {
                        b'n' => match r[4] {
                            b'y' => match r[5] {
                                b'm' => match r[6] {
                                    b'o' => match r[7] {
                                        b'u' => match r[8] {
                                            b's' => s(Anonymous),
                                            _ => N,
                                        },
                                        _ => N,
                                    },
                                    _ => N,
                                },
                                _ => N,
                            },
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'p' => match r[1] {
                b'r' => match r[2] {
                    b'o' => match r[3] {
                        b't' => match r[4] {
                            b'e' => match r[5] {
                                b'c' => match r[6] {
                                    b't' => match r[7] {
                                        b'e' => match r[8] {
                                            b'd' => s(Protected),
                                            _ => N,
                                        },
                                        _ => N,
                                    },
                                    _ => N,
                                },
                                _ => N,
                            },
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            _ => N,
        },
        10 => match r[0] {
            b'e' => match r[1] {
                b'x' => match r[2] {
                    b'e' => match r[3] {
                        b'c' => match r[4] {
                            b'u' => match r[5] {
                                b't' => match r[6] {
                                    b'a' => match r[7] {
                                        b'b' => match r[8] {
                                            b'l' => match r[9] {
                                                b'e' => s(Executable),
                                                _ => N,
                                            },
                                            _ => N,
                                        },
                                        _ => N,
                                    },
                                    _ => N,
                                },
                                _ => N,
                            },
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            _ => N,
        },
        _ => N,
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Keyword {
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}
