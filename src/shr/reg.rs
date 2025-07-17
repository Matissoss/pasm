// pasm - src/shr/reg.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::size::Size;
use std::str::FromStr;

#[allow(clippy::to_string_trait_impl)]
impl ToString for Purpose {
    fn to_string(&self) -> String {
        match self {
            Self::General => "r".to_string(),
            Self::Mmx => "mm".to_string(),
            Self::F128 => "xmm".to_string(),
            Self::F256 => "ymm".to_string(),
            Self::Sgmnt => "segment".to_string(),
            Self::IPtr => "ip".to_string(),
            Self::Dbg => "dr".to_string(),
            Self::Ctrl => "cr".to_string(),
            Self::__ANY => "any".to_string(),
            Self::F512 => "zmm".to_string(),
            Self::Mask => "k".to_string(),
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Register {
    fn to_string(&self) -> String {
        "".to_string()
    }
}

impl FromStr for Register {
    type Err = ();
    fn from_str(str: &str) -> Result<Register, ()> {
        if let Some(r) = reg_fromstr(str) {
            Ok(r)
        } else {
            Err(())
        }
    }
}

#[inline(always)]
fn s<T>(t: T) -> Option<T> {
    Some(t)
}

const N: Option<Register> = None;

pub fn reg_fromstr(str: &str) -> Option<Register> {
    let r = str.as_bytes();
    match r.len() {
        2 => match r[0] {
            b'e' => match r[1] {
                b's' => s(Register::ES),
                _ => N,
            },
            b'f' => match r[1] {
                b's' => s(Register::FS),
                _ => N,
            },
            b'g' => match r[1] {
                b's' => s(Register::GS),
                _ => N,
            },
            b'i' => match r[1] {
                b'p' => s(Register::IP),
                _ => N,
            },
            b'a' => match r[1] {
                b'h' => s(Register::AH),
                b'l' => s(Register::AL),
                b'x' => s(Register::AX),
                _ => N,
            },
            b'b' => match r[1] {
                b'h' => s(Register::BH),
                b'l' => s(Register::BL),
                b'p' => s(Register::BP),
                b'x' => s(Register::BX),
                _ => N,
            },
            b'c' => match r[1] {
                b'h' => s(Register::CH),
                b'l' => s(Register::CL),
                b's' => s(Register::CS),
                b'x' => s(Register::CX),
                _ => N,
            },
            b'd' => match r[1] {
                b'h' => s(Register::DH),
                b'i' => s(Register::DI),
                b'l' => s(Register::DL),
                b's' => s(Register::DS),
                b'x' => s(Register::DX),
                _ => N,
            },
            b'k' => match r[1] {
                b'0' => s(Register::K0),
                b'1' => s(Register::K1),
                b'2' => s(Register::K2),
                b'3' => s(Register::K3),
                b'4' => s(Register::K4),
                b'5' => s(Register::K5),
                b'6' => s(Register::K6),
                b'7' => s(Register::K7),
                _ => N,
            },
            b'r' => match r[1] {
                b'8' => s(Register::R8),
                b'9' => s(Register::R9),
                _ => N,
            },
            b's' => match r[1] {
                b'i' => s(Register::SI),
                b'p' => s(Register::SP),
                b's' => s(Register::SS),
                _ => N,
            },
            _ => N,
        },
        3 => match r[0] {
            b'b' => match r[1] {
                b'p' => match r[2] {
                    b'l' => s(Register::BPL),
                    _ => N,
                },
                _ => N,
            },
            b'c' => match r[1] {
                b'r' => match r[2] {
                    b'0' => s(Register::CR0),
                    b'1' => s(Register::CR1),
                    b'2' => s(Register::CR2),
                    b'3' => s(Register::CR3),
                    b'4' => s(Register::CR4),
                    b'5' => s(Register::CR5),
                    b'6' => s(Register::CR6),
                    b'7' => s(Register::CR7),
                    b'8' => s(Register::CR8),
                    b'9' => s(Register::CR9),
                    _ => N,
                },
                _ => N,
            },
            b'm' => match r[1] {
                b'm' => match r[2] {
                    b'0' => s(Register::MM0),
                    b'1' => s(Register::MM1),
                    b'2' => s(Register::MM2),
                    b'3' => s(Register::MM3),
                    b'4' => s(Register::MM4),
                    b'5' => s(Register::MM5),
                    b'6' => s(Register::MM6),
                    b'7' => s(Register::MM7),
                    _ => N,
                },
                _ => N,
            },
            b'd' => match r[1] {
                b'i' => match r[2] {
                    b'l' => s(Register::DIL),
                    _ => N,
                },
                b'r' => match r[2] {
                    b'0' => s(Register::DR0),
                    b'1' => s(Register::DR1),
                    b'2' => s(Register::DR2),
                    b'3' => s(Register::DR3),
                    b'4' => s(Register::DR4),
                    b'5' => s(Register::DR5),
                    b'6' => s(Register::DR6),
                    b'7' => s(Register::DR7),
                    b'8' => s(Register::DR8),
                    b'9' => s(Register::DR9),
                    _ => N,
                },
                _ => N,
            },
            b'e' => match r[1] {
                b'a' => match r[2] {
                    b'x' => s(Register::EAX),
                    _ => N,
                },
                b'c' => match r[2] {
                    b'x' => s(Register::ECX),
                    _ => N,
                },
                b'i' => match r[2] {
                    b'p' => s(Register::EIP),
                    _ => N,
                },
                b'b' => match r[2] {
                    b'p' => s(Register::EBP),
                    b'x' => s(Register::EBX),
                    _ => N,
                },
                b'd' => match r[2] {
                    b'i' => s(Register::EDI),
                    b'x' => s(Register::EDX),
                    _ => N,
                },
                b's' => match r[2] {
                    b'i' => s(Register::ESI),
                    b'p' => s(Register::ESP),
                    _ => N,
                },
                _ => N,
            },
            b'r' => match r[1] {
                b'a' => match r[2] {
                    b'x' => s(Register::RAX),
                    _ => N,
                },
                b'c' => match r[2] {
                    b'x' => s(Register::RCX),
                    _ => N,
                },
                b'i' => match r[2] {
                    b'p' => s(Register::RIP),
                    _ => N,
                },
                b'1' => match r[2] {
                    b'0' => s(Register::R10),
                    b'1' => s(Register::R11),
                    b'2' => s(Register::R12),
                    b'3' => s(Register::R13),
                    b'4' => s(Register::R14),
                    b'5' => s(Register::R15),
                    _ => N,
                },
                b'8' => match r[2] {
                    b'b' => s(Register::R8B),
                    b'd' => s(Register::R8D),
                    b'w' => s(Register::R8W),
                    _ => N,
                },
                b'9' => match r[2] {
                    b'b' => s(Register::R9B),
                    b'd' => s(Register::R9D),
                    b'w' => s(Register::R9W),
                    _ => N,
                },
                b'b' => match r[2] {
                    b'p' => s(Register::RBP),
                    b'x' => s(Register::RBX),
                    _ => N,
                },
                b'd' => match r[2] {
                    b'i' => s(Register::RDI),
                    b'x' => s(Register::RDX),
                    _ => N,
                },
                b's' => match r[2] {
                    b'i' => s(Register::RSI),
                    b'p' => s(Register::RSP),
                    _ => N,
                },
                _ => N,
            },
            b's' => match r[1] {
                b'i' => match r[2] {
                    b'l' => s(Register::SIL),
                    _ => N,
                },
                b'p' => match r[2] {
                    b'l' => s(Register::SPL),
                    _ => N,
                },
                _ => N,
            },
            _ => N,
        },
        4 => match r[0] {
            b'c' => match r[1] {
                b'r' => match r[2] {
                    b'1' => match r[3] {
                        b'0' => s(Register::CR10),
                        b'1' => s(Register::CR11),
                        b'2' => s(Register::CR12),
                        b'3' => s(Register::CR13),
                        b'4' => s(Register::CR14),
                        b'5' => s(Register::CR15),
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'd' => match r[1] {
                b'r' => match r[2] {
                    b'1' => match r[3] {
                        b'0' => s(Register::DR10),
                        b'1' => s(Register::DR11),
                        b'2' => s(Register::DR12),
                        b'3' => s(Register::DR13),
                        b'4' => s(Register::DR14),
                        b'5' => s(Register::DR15),
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'r' => match r[1] {
                b'1' => match r[2] {
                    b'0' => match r[3] {
                        b'b' => s(Register::R10B),
                        b'd' => s(Register::R10D),
                        b'w' => s(Register::R10W),
                        _ => N,
                    },
                    b'1' => match r[3] {
                        b'b' => s(Register::R11B),
                        b'd' => s(Register::R11D),
                        b'w' => s(Register::R11W),
                        _ => N,
                    },
                    b'2' => match r[3] {
                        b'b' => s(Register::R12B),
                        b'd' => s(Register::R12D),
                        b'w' => s(Register::R12W),
                        _ => N,
                    },
                    b'3' => match r[3] {
                        b'b' => s(Register::R13B),
                        b'd' => s(Register::R13D),
                        b'w' => s(Register::R13W),
                        _ => N,
                    },
                    b'4' => match r[3] {
                        b'b' => s(Register::R14B),
                        b'd' => s(Register::R14D),
                        b'w' => s(Register::R14W),
                        _ => N,
                    },
                    b'5' => match r[3] {
                        b'b' => s(Register::R15B),
                        b'd' => s(Register::R15D),
                        b'w' => s(Register::R15W),
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'x' => match r[1] {
                b'm' => match r[2] {
                    b'm' => match r[3] {
                        b'0' => s(Register::XMM0),
                        b'1' => s(Register::XMM1),
                        b'2' => s(Register::XMM2),
                        b'3' => s(Register::XMM3),
                        b'4' => s(Register::XMM4),
                        b'5' => s(Register::XMM5),
                        b'6' => s(Register::XMM6),
                        b'7' => s(Register::XMM7),
                        b'8' => s(Register::XMM8),
                        b'9' => s(Register::XMM9),
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'y' => match r[1] {
                b'm' => match r[2] {
                    b'm' => match r[3] {
                        b'0' => s(Register::YMM0),
                        b'1' => s(Register::YMM1),
                        b'2' => s(Register::YMM2),
                        b'3' => s(Register::YMM3),
                        b'4' => s(Register::YMM4),
                        b'5' => s(Register::YMM5),
                        b'6' => s(Register::YMM6),
                        b'7' => s(Register::YMM7),
                        b'8' => s(Register::YMM8),
                        b'9' => s(Register::YMM9),
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'z' => match r[1] {
                b'm' => match r[2] {
                    b'm' => match r[3] {
                        b'0' => s(Register::ZMM0),
                        b'1' => s(Register::ZMM1),
                        b'2' => s(Register::ZMM2),
                        b'3' => s(Register::ZMM3),
                        b'4' => s(Register::ZMM4),
                        b'5' => s(Register::ZMM5),
                        b'6' => s(Register::ZMM6),
                        b'7' => s(Register::ZMM7),
                        b'8' => s(Register::ZMM8),
                        b'9' => s(Register::ZMM9),
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            _ => N,
        },
        5 => match r[0] {
            b'x' => match r[1] {
                b'm' => match r[2] {
                    b'm' => match r[3] {
                        b'1' => match r[4] {
                            b'0' => s(Register::XMM10),
                            b'1' => s(Register::XMM11),
                            b'2' => s(Register::XMM12),
                            b'3' => s(Register::XMM13),
                            b'4' => s(Register::XMM14),
                            b'5' => s(Register::XMM15),
                            b'6' => s(Register::XMM16),
                            b'7' => s(Register::XMM17),
                            b'8' => s(Register::XMM18),
                            b'9' => s(Register::XMM19),
                            _ => N,
                        },
                        b'2' => match r[4] {
                            b'0' => s(Register::XMM20),
                            b'1' => s(Register::XMM21),
                            b'2' => s(Register::XMM22),
                            b'3' => s(Register::XMM23),
                            b'4' => s(Register::XMM24),
                            b'5' => s(Register::XMM25),
                            b'6' => s(Register::XMM26),
                            b'7' => s(Register::XMM27),
                            b'8' => s(Register::XMM28),
                            b'9' => s(Register::XMM29),
                            _ => N,
                        },
                        b'3' => match r[4] {
                            b'0' => s(Register::XMM30),
                            b'1' => s(Register::XMM31),
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'y' => match r[1] {
                b'm' => match r[2] {
                    b'm' => match r[3] {
                        b'1' => match r[4] {
                            b'0' => s(Register::YMM10),
                            b'1' => s(Register::YMM11),
                            b'2' => s(Register::YMM12),
                            b'3' => s(Register::YMM13),
                            b'4' => s(Register::YMM14),
                            b'5' => s(Register::YMM15),
                            b'6' => s(Register::YMM16),
                            b'7' => s(Register::YMM17),
                            b'8' => s(Register::YMM18),
                            b'9' => s(Register::YMM19),
                            _ => N,
                        },
                        b'2' => match r[4] {
                            b'0' => s(Register::YMM20),
                            b'1' => s(Register::YMM21),
                            b'2' => s(Register::YMM22),
                            b'3' => s(Register::YMM23),
                            b'4' => s(Register::YMM24),
                            b'5' => s(Register::YMM25),
                            b'6' => s(Register::YMM26),
                            b'7' => s(Register::YMM27),
                            b'8' => s(Register::YMM28),
                            b'9' => s(Register::YMM29),
                            _ => N,
                        },
                        b'3' => match r[4] {
                            b'0' => s(Register::YMM30),
                            b'1' => s(Register::YMM31),
                            _ => N,
                        },
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'z' => match r[1] {
                b'm' => match r[2] {
                    b'm' => match r[3] {
                        b'1' => match r[4] {
                            b'0' => s(Register::ZMM10),
                            b'1' => s(Register::ZMM11),
                            b'2' => s(Register::ZMM12),
                            b'3' => s(Register::ZMM13),
                            b'4' => s(Register::ZMM14),
                            b'5' => s(Register::ZMM15),
                            b'6' => s(Register::ZMM16),
                            b'7' => s(Register::ZMM17),
                            b'8' => s(Register::ZMM18),
                            b'9' => s(Register::ZMM19),
                            _ => N,
                        },
                        b'2' => match r[4] {
                            b'0' => s(Register::ZMM20),
                            b'1' => s(Register::ZMM21),
                            b'2' => s(Register::ZMM22),
                            b'3' => s(Register::ZMM23),
                            b'4' => s(Register::ZMM24),
                            b'5' => s(Register::ZMM25),
                            b'6' => s(Register::ZMM26),
                            b'7' => s(Register::ZMM27),
                            b'8' => s(Register::ZMM28),
                            b'9' => s(Register::ZMM29),
                            _ => N,
                        },
                        b'3' => match r[4] {
                            b'0' => s(Register::ZMM30),
                            b'1' => s(Register::ZMM31),
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

impl Purpose {
    pub const fn is_avx(&self) -> bool {
        *self as u8 == Self::F128 as u8
            || *self as u8 == Self::F256 as u8
            || *self as u8 == Self::F512 as u8
    }
    pub const fn is_gpr(&self) -> bool {
        *self as u8 == Self::General as u8
    }
    pub const fn is_mask(&self) -> bool {
        *self as u8 == Self::Mask as u8
    }
    pub const fn is_any(&self) -> bool {
        *self as u8 == Self::__ANY as u8
    }
    pub const fn is_ctrl(&self) -> bool {
        *self as u8 == Self::Ctrl as u8
    }
    pub const fn is_sgmnt(&self) -> bool {
        *self as u8 == Self::Sgmnt as u8
    }
    pub const fn is_dbg(&self) -> bool {
        *self as u8 == Self::Dbg as u8
    }
}

impl PartialEq for Purpose {
    fn eq(&self, rhs: &Self) -> bool {
        let su8 = *self as u8;
        let ru8 = *rhs as u8;
        if su8 == Self::__ANY as u8 || ru8 == Self::__ANY as u8 {
            true
        } else {
            su8 == ru8
        }
    }
}

#[repr(u8)]
#[derive(Debug, Eq, Clone, Copy)]
pub enum Purpose {
    __ANY = 0,
    General = 1,
    IPtr = 2,   // ip/rip/eip
    Dbg = 3,    // drX
    Ctrl = 4,   // crX
    Mask = 5,   // k0-7
    Mmx = 6,    // mmX
    F128 = 7,   // xmmX
    F256 = 8,   // ymmX
    F512 = 9,   // zmmX
    Sgmnt = 10, // segment registers (cs, ss, ds, es, ...)
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Debug)]
// layout:
//  0b000P_PPPS_SSSE_ECCC
pub struct Register(pub u16);

impl Register {
    pub const ES: Self = Self::new(Purpose::Sgmnt, Size::Word, [false, false], 0b000);
    pub const CS: Self = Self::new(Purpose::Sgmnt, Size::Word, [false, false], 0b001);
    pub const SS: Self = Self::new(Purpose::Sgmnt, Size::Word, [false, false], 0b010);
    pub const DS: Self = Self::new(Purpose::Sgmnt, Size::Word, [false, false], 0b011);
    pub const FS: Self = Self::new(Purpose::Sgmnt, Size::Word, [false, false], 0b100);
    pub const GS: Self = Self::new(Purpose::Sgmnt, Size::Word, [false, false], 0b101);

    pub const AL: Self = Self::new(Purpose::General, Size::Byte, [false, false], 0b000);
    pub const CL: Self = Self::new(Purpose::General, Size::Byte, [false, false], 0b001);
    pub const DL: Self = Self::new(Purpose::General, Size::Byte, [false, false], 0b010);
    pub const BL: Self = Self::new(Purpose::General, Size::Byte, [false, false], 0b011);
    pub const AH: Self = Self::new(Purpose::General, Size::Byte, [false, false], 0b100);
    pub const CH: Self = Self::new(Purpose::General, Size::Byte, [false, false], 0b101);
    pub const DH: Self = Self::new(Purpose::General, Size::Byte, [false, false], 0b110);
    pub const BH: Self = Self::new(Purpose::General, Size::Byte, [false, false], 0b111);
    pub const SPL: Self = Self::new(Purpose::General, Size::Byte, [false, false], 0b100);
    pub const BPL: Self = Self::new(Purpose::General, Size::Byte, [false, false], 0b101);
    pub const SIL: Self = Self::new(Purpose::General, Size::Byte, [false, false], 0b110);
    pub const DIL: Self = Self::new(Purpose::General, Size::Byte, [false, false], 0b111);

    pub const R8B: Self = Self::new(Purpose::General, Size::Byte, [false, true], 0b000);
    pub const R9B: Self = Self::new(Purpose::General, Size::Byte, [false, true], 0b001);
    pub const R10B: Self = Self::new(Purpose::General, Size::Byte, [false, true], 0b010);
    pub const R11B: Self = Self::new(Purpose::General, Size::Byte, [false, true], 0b011);
    pub const R12B: Self = Self::new(Purpose::General, Size::Byte, [false, true], 0b100);
    pub const R13B: Self = Self::new(Purpose::General, Size::Byte, [false, true], 0b101);
    pub const R14B: Self = Self::new(Purpose::General, Size::Byte, [false, true], 0b110);
    pub const R15B: Self = Self::new(Purpose::General, Size::Byte, [false, true], 0b111);

    pub const AX: Self = Self::new(Purpose::General, Size::Word, [false, false], 0b000);
    pub const CX: Self = Self::new(Purpose::General, Size::Word, [false, false], 0b001);
    pub const DX: Self = Self::new(Purpose::General, Size::Word, [false, false], 0b010);
    pub const BX: Self = Self::new(Purpose::General, Size::Word, [false, false], 0b011);
    pub const SP: Self = Self::new(Purpose::General, Size::Word, [false, false], 0b100);
    pub const BP: Self = Self::new(Purpose::General, Size::Word, [false, false], 0b101);
    pub const SI: Self = Self::new(Purpose::General, Size::Word, [false, false], 0b110);
    pub const DI: Self = Self::new(Purpose::General, Size::Word, [false, false], 0b111);

    pub const R8W: Self = Self::new(Purpose::General, Size::Word, [false, true], 0b000);
    pub const R9W: Self = Self::new(Purpose::General, Size::Word, [false, true], 0b001);
    pub const R10W: Self = Self::new(Purpose::General, Size::Word, [false, true], 0b010);
    pub const R11W: Self = Self::new(Purpose::General, Size::Word, [false, true], 0b011);
    pub const R12W: Self = Self::new(Purpose::General, Size::Word, [false, true], 0b100);
    pub const R13W: Self = Self::new(Purpose::General, Size::Word, [false, true], 0b101);
    pub const R14W: Self = Self::new(Purpose::General, Size::Word, [false, true], 0b110);
    pub const R15W: Self = Self::new(Purpose::General, Size::Word, [false, true], 0b111);

    pub const EAX: Self = Self::new(Purpose::General, Size::Dword, [false, false], 0b000);
    pub const ECX: Self = Self::new(Purpose::General, Size::Dword, [false, false], 0b001);
    pub const EDX: Self = Self::new(Purpose::General, Size::Dword, [false, false], 0b010);
    pub const EBX: Self = Self::new(Purpose::General, Size::Dword, [false, false], 0b011);
    pub const ESP: Self = Self::new(Purpose::General, Size::Dword, [false, false], 0b100);
    pub const EBP: Self = Self::new(Purpose::General, Size::Dword, [false, false], 0b101);
    pub const ESI: Self = Self::new(Purpose::General, Size::Dword, [false, false], 0b110);
    pub const EDI: Self = Self::new(Purpose::General, Size::Dword, [false, false], 0b111);

    pub const R8D: Self = Self::new(Purpose::General, Size::Dword, [false, true], 0b000);
    pub const R9D: Self = Self::new(Purpose::General, Size::Dword, [false, true], 0b001);
    pub const R10D: Self = Self::new(Purpose::General, Size::Dword, [false, true], 0b010);
    pub const R11D: Self = Self::new(Purpose::General, Size::Dword, [false, true], 0b011);
    pub const R12D: Self = Self::new(Purpose::General, Size::Dword, [false, true], 0b100);
    pub const R13D: Self = Self::new(Purpose::General, Size::Dword, [false, true], 0b101);
    pub const R14D: Self = Self::new(Purpose::General, Size::Dword, [false, true], 0b110);
    pub const R15D: Self = Self::new(Purpose::General, Size::Dword, [false, true], 0b111);

    pub const RAX: Self = Self::new(Purpose::General, Size::Qword, [false, false], 0b000);
    pub const RCX: Self = Self::new(Purpose::General, Size::Qword, [false, false], 0b001);
    pub const RDX: Self = Self::new(Purpose::General, Size::Qword, [false, false], 0b010);
    pub const RBX: Self = Self::new(Purpose::General, Size::Qword, [false, false], 0b011);
    pub const RSP: Self = Self::new(Purpose::General, Size::Qword, [false, false], 0b100);
    pub const RBP: Self = Self::new(Purpose::General, Size::Qword, [false, false], 0b101);
    pub const RSI: Self = Self::new(Purpose::General, Size::Qword, [false, false], 0b110);
    pub const RDI: Self = Self::new(Purpose::General, Size::Qword, [false, false], 0b111);

    pub const R8: Self = Self::new(Purpose::General, Size::Qword, [false, true], 0b000);
    pub const R9: Self = Self::new(Purpose::General, Size::Qword, [false, true], 0b001);
    pub const R10: Self = Self::new(Purpose::General, Size::Qword, [false, true], 0b010);
    pub const R11: Self = Self::new(Purpose::General, Size::Qword, [false, true], 0b011);
    pub const R12: Self = Self::new(Purpose::General, Size::Qword, [false, true], 0b100);
    pub const R13: Self = Self::new(Purpose::General, Size::Qword, [false, true], 0b101);
    pub const R14: Self = Self::new(Purpose::General, Size::Qword, [false, true], 0b110);
    pub const R15: Self = Self::new(Purpose::General, Size::Qword, [false, true], 0b111);

    pub const K0: Self = Self::new(Purpose::Mask, Size::Qword, [false, false], 0b000);
    pub const K1: Self = Self::new(Purpose::Mask, Size::Qword, [false, false], 0b001);
    pub const K2: Self = Self::new(Purpose::Mask, Size::Qword, [false, false], 0b010);
    pub const K3: Self = Self::new(Purpose::Mask, Size::Qword, [false, false], 0b011);
    pub const K4: Self = Self::new(Purpose::Mask, Size::Qword, [false, false], 0b100);
    pub const K5: Self = Self::new(Purpose::Mask, Size::Qword, [false, false], 0b101);
    pub const K6: Self = Self::new(Purpose::Mask, Size::Qword, [false, false], 0b110);
    pub const K7: Self = Self::new(Purpose::Mask, Size::Qword, [false, false], 0b111);

    pub const MM0: Self = Self::new(Purpose::Mmx, Size::Qword, [false, false], 0b000);
    pub const MM1: Self = Self::new(Purpose::Mmx, Size::Qword, [false, false], 0b001);
    pub const MM2: Self = Self::new(Purpose::Mmx, Size::Qword, [false, false], 0b010);
    pub const MM3: Self = Self::new(Purpose::Mmx, Size::Qword, [false, false], 0b011);
    pub const MM4: Self = Self::new(Purpose::Mmx, Size::Qword, [false, false], 0b100);
    pub const MM5: Self = Self::new(Purpose::Mmx, Size::Qword, [false, false], 0b101);
    pub const MM6: Self = Self::new(Purpose::Mmx, Size::Qword, [false, false], 0b110);
    pub const MM7: Self = Self::new(Purpose::Mmx, Size::Qword, [false, false], 0b111);

    pub const XMM0: Self = Self::new(Purpose::F128, Size::Xword, [false, false], 0b000);
    pub const XMM1: Self = Self::new(Purpose::F128, Size::Xword, [false, false], 0b001);
    pub const XMM2: Self = Self::new(Purpose::F128, Size::Xword, [false, false], 0b010);
    pub const XMM3: Self = Self::new(Purpose::F128, Size::Xword, [false, false], 0b011);
    pub const XMM4: Self = Self::new(Purpose::F128, Size::Xword, [false, false], 0b100);
    pub const XMM5: Self = Self::new(Purpose::F128, Size::Xword, [false, false], 0b101);
    pub const XMM6: Self = Self::new(Purpose::F128, Size::Xword, [false, false], 0b110);
    pub const XMM7: Self = Self::new(Purpose::F128, Size::Xword, [false, false], 0b111);
    pub const XMM8: Self = Self::new(Purpose::F128, Size::Xword, [false, true], 0b000);
    pub const XMM9: Self = Self::new(Purpose::F128, Size::Xword, [false, true], 0b001);
    pub const XMM10: Self = Self::new(Purpose::F128, Size::Xword, [false, true], 0b010);
    pub const XMM11: Self = Self::new(Purpose::F128, Size::Xword, [false, true], 0b011);
    pub const XMM12: Self = Self::new(Purpose::F128, Size::Xword, [false, true], 0b100);
    pub const XMM13: Self = Self::new(Purpose::F128, Size::Xword, [false, true], 0b101);
    pub const XMM14: Self = Self::new(Purpose::F128, Size::Xword, [false, true], 0b110);
    pub const XMM15: Self = Self::new(Purpose::F128, Size::Xword, [false, true], 0b111);

    pub const XMM16: Self = Self::new(Purpose::F128, Size::Xword, [true, false], 0b000);
    pub const XMM17: Self = Self::new(Purpose::F128, Size::Xword, [true, false], 0b001);
    pub const XMM18: Self = Self::new(Purpose::F128, Size::Xword, [true, false], 0b010);
    pub const XMM19: Self = Self::new(Purpose::F128, Size::Xword, [true, false], 0b011);
    pub const XMM20: Self = Self::new(Purpose::F128, Size::Xword, [true, false], 0b100);
    pub const XMM21: Self = Self::new(Purpose::F128, Size::Xword, [true, false], 0b101);
    pub const XMM22: Self = Self::new(Purpose::F128, Size::Xword, [true, false], 0b110);
    pub const XMM23: Self = Self::new(Purpose::F128, Size::Xword, [true, false], 0b111);
    pub const XMM24: Self = Self::new(Purpose::F128, Size::Xword, [true, true], 0b000);
    pub const XMM25: Self = Self::new(Purpose::F128, Size::Xword, [true, true], 0b001);
    pub const XMM26: Self = Self::new(Purpose::F128, Size::Xword, [true, true], 0b010);
    pub const XMM27: Self = Self::new(Purpose::F128, Size::Xword, [true, true], 0b011);
    pub const XMM28: Self = Self::new(Purpose::F128, Size::Xword, [true, true], 0b100);
    pub const XMM29: Self = Self::new(Purpose::F128, Size::Xword, [true, true], 0b101);
    pub const XMM30: Self = Self::new(Purpose::F128, Size::Xword, [true, true], 0b110);
    pub const XMM31: Self = Self::new(Purpose::F128, Size::Xword, [true, true], 0b111);

    pub const YMM0: Self = Self::new(Purpose::F256, Size::Yword, [false, false], 0b000);
    pub const YMM1: Self = Self::new(Purpose::F256, Size::Yword, [false, false], 0b001);
    pub const YMM2: Self = Self::new(Purpose::F256, Size::Yword, [false, false], 0b010);
    pub const YMM3: Self = Self::new(Purpose::F256, Size::Yword, [false, false], 0b011);
    pub const YMM4: Self = Self::new(Purpose::F256, Size::Yword, [false, false], 0b100);
    pub const YMM5: Self = Self::new(Purpose::F256, Size::Yword, [false, false], 0b101);
    pub const YMM6: Self = Self::new(Purpose::F256, Size::Yword, [false, false], 0b110);
    pub const YMM7: Self = Self::new(Purpose::F256, Size::Yword, [false, false], 0b111);
    pub const YMM8: Self = Self::new(Purpose::F256, Size::Yword, [false, true], 0b000);
    pub const YMM9: Self = Self::new(Purpose::F256, Size::Yword, [false, true], 0b001);
    pub const YMM10: Self = Self::new(Purpose::F256, Size::Yword, [false, true], 0b010);
    pub const YMM11: Self = Self::new(Purpose::F256, Size::Yword, [false, true], 0b011);
    pub const YMM12: Self = Self::new(Purpose::F256, Size::Yword, [false, true], 0b100);
    pub const YMM13: Self = Self::new(Purpose::F256, Size::Yword, [false, true], 0b101);
    pub const YMM14: Self = Self::new(Purpose::F256, Size::Yword, [false, true], 0b110);
    pub const YMM15: Self = Self::new(Purpose::F256, Size::Yword, [false, true], 0b111);

    pub const YMM16: Self = Self::new(Purpose::F256, Size::Yword, [true, false], 0b000);
    pub const YMM17: Self = Self::new(Purpose::F256, Size::Yword, [true, false], 0b001);
    pub const YMM18: Self = Self::new(Purpose::F256, Size::Yword, [true, false], 0b010);
    pub const YMM19: Self = Self::new(Purpose::F256, Size::Yword, [true, false], 0b011);
    pub const YMM20: Self = Self::new(Purpose::F256, Size::Yword, [true, false], 0b100);
    pub const YMM21: Self = Self::new(Purpose::F256, Size::Yword, [true, false], 0b101);
    pub const YMM22: Self = Self::new(Purpose::F256, Size::Yword, [true, false], 0b110);
    pub const YMM23: Self = Self::new(Purpose::F256, Size::Yword, [true, false], 0b111);
    pub const YMM24: Self = Self::new(Purpose::F256, Size::Yword, [true, true], 0b000);
    pub const YMM25: Self = Self::new(Purpose::F256, Size::Yword, [true, true], 0b001);
    pub const YMM26: Self = Self::new(Purpose::F256, Size::Yword, [true, true], 0b010);
    pub const YMM27: Self = Self::new(Purpose::F256, Size::Yword, [true, true], 0b011);
    pub const YMM28: Self = Self::new(Purpose::F256, Size::Yword, [true, true], 0b100);
    pub const YMM29: Self = Self::new(Purpose::F256, Size::Yword, [true, true], 0b101);
    pub const YMM30: Self = Self::new(Purpose::F256, Size::Yword, [true, true], 0b110);
    pub const YMM31: Self = Self::new(Purpose::F256, Size::Yword, [true, true], 0b111);

    pub const ZMM0: Self = Self::new(Purpose::F512, Size::Zword, [false, false], 0b000);
    pub const ZMM1: Self = Self::new(Purpose::F512, Size::Zword, [false, false], 0b001);
    pub const ZMM2: Self = Self::new(Purpose::F512, Size::Zword, [false, false], 0b010);
    pub const ZMM3: Self = Self::new(Purpose::F512, Size::Zword, [false, false], 0b011);
    pub const ZMM4: Self = Self::new(Purpose::F512, Size::Zword, [false, false], 0b100);
    pub const ZMM5: Self = Self::new(Purpose::F512, Size::Zword, [false, false], 0b101);
    pub const ZMM6: Self = Self::new(Purpose::F512, Size::Zword, [false, false], 0b110);
    pub const ZMM7: Self = Self::new(Purpose::F512, Size::Zword, [false, false], 0b111);
    pub const ZMM8: Self = Self::new(Purpose::F512, Size::Zword, [false, true], 0b000);
    pub const ZMM9: Self = Self::new(Purpose::F512, Size::Zword, [false, true], 0b001);
    pub const ZMM10: Self = Self::new(Purpose::F512, Size::Zword, [false, true], 0b010);
    pub const ZMM11: Self = Self::new(Purpose::F512, Size::Zword, [false, true], 0b011);
    pub const ZMM12: Self = Self::new(Purpose::F512, Size::Zword, [false, true], 0b100);
    pub const ZMM13: Self = Self::new(Purpose::F512, Size::Zword, [false, true], 0b101);
    pub const ZMM14: Self = Self::new(Purpose::F512, Size::Zword, [false, true], 0b110);
    pub const ZMM15: Self = Self::new(Purpose::F512, Size::Zword, [false, true], 0b111);

    pub const ZMM16: Self = Self::new(Purpose::F512, Size::Zword, [true, false], 0b000);
    pub const ZMM17: Self = Self::new(Purpose::F512, Size::Zword, [true, false], 0b001);
    pub const ZMM18: Self = Self::new(Purpose::F512, Size::Zword, [true, false], 0b010);
    pub const ZMM19: Self = Self::new(Purpose::F512, Size::Zword, [true, false], 0b011);
    pub const ZMM20: Self = Self::new(Purpose::F512, Size::Zword, [true, false], 0b100);
    pub const ZMM21: Self = Self::new(Purpose::F512, Size::Zword, [true, false], 0b101);
    pub const ZMM22: Self = Self::new(Purpose::F512, Size::Zword, [true, false], 0b110);
    pub const ZMM23: Self = Self::new(Purpose::F512, Size::Zword, [true, false], 0b111);
    pub const ZMM24: Self = Self::new(Purpose::F512, Size::Zword, [true, true], 0b000);
    pub const ZMM25: Self = Self::new(Purpose::F512, Size::Zword, [true, true], 0b001);
    pub const ZMM26: Self = Self::new(Purpose::F512, Size::Zword, [true, true], 0b010);
    pub const ZMM27: Self = Self::new(Purpose::F512, Size::Zword, [true, true], 0b011);
    pub const ZMM28: Self = Self::new(Purpose::F512, Size::Zword, [true, true], 0b100);
    pub const ZMM29: Self = Self::new(Purpose::F512, Size::Zword, [true, true], 0b101);
    pub const ZMM30: Self = Self::new(Purpose::F512, Size::Zword, [true, true], 0b110);
    pub const ZMM31: Self = Self::new(Purpose::F512, Size::Zword, [true, true], 0b111);

    pub const CR0: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, false], 0b000);
    pub const CR1: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, false], 0b001);
    pub const CR2: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, false], 0b010);
    pub const CR3: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, false], 0b011);
    pub const CR4: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, false], 0b100);
    pub const CR5: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, false], 0b101);
    pub const CR6: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, false], 0b110);
    pub const CR7: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, false], 0b111);
    pub const CR8: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, true], 0b000);
    pub const CR9: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, true], 0b001);
    pub const CR10: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, true], 0b010);
    pub const CR11: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, true], 0b011);
    pub const CR12: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, true], 0b100);
    pub const CR13: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, true], 0b101);
    pub const CR14: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, true], 0b110);
    pub const CR15: Self = Self::new(Purpose::Ctrl, Size::Dword, [false, true], 0b111);

    pub const DR0: Self = Self::new(Purpose::Dbg, Size::Dword, [false, false], 0b000);
    pub const DR1: Self = Self::new(Purpose::Dbg, Size::Dword, [false, false], 0b001);
    pub const DR2: Self = Self::new(Purpose::Dbg, Size::Dword, [false, false], 0b010);
    pub const DR3: Self = Self::new(Purpose::Dbg, Size::Dword, [false, false], 0b011);
    pub const DR4: Self = Self::new(Purpose::Dbg, Size::Dword, [false, false], 0b100);
    pub const DR5: Self = Self::new(Purpose::Dbg, Size::Dword, [false, false], 0b101);
    pub const DR6: Self = Self::new(Purpose::Dbg, Size::Dword, [false, false], 0b110);
    pub const DR7: Self = Self::new(Purpose::Dbg, Size::Dword, [false, false], 0b111);
    pub const DR8: Self = Self::new(Purpose::Dbg, Size::Dword, [false, true], 0b000);
    pub const DR9: Self = Self::new(Purpose::Dbg, Size::Dword, [false, true], 0b001);
    pub const DR10: Self = Self::new(Purpose::Dbg, Size::Dword, [false, true], 0b010);
    pub const DR11: Self = Self::new(Purpose::Dbg, Size::Dword, [false, true], 0b011);
    pub const DR12: Self = Self::new(Purpose::Dbg, Size::Dword, [false, true], 0b100);
    pub const DR13: Self = Self::new(Purpose::Dbg, Size::Dword, [false, true], 0b101);
    pub const DR14: Self = Self::new(Purpose::Dbg, Size::Dword, [false, true], 0b110);
    pub const DR15: Self = Self::new(Purpose::Dbg, Size::Dword, [false, true], 0b111);

    pub const RIP: Self = Self::new(Purpose::IPtr, Size::Qword, [false, false], 0b000);
    pub const EIP: Self = Self::new(Purpose::IPtr, Size::Dword, [false, false], 0b000);
    pub const IP: Self = Self::new(Purpose::IPtr, Size::Word, [false, false], 0b000);

    pub const __ANY: Self = Self::new(Purpose::__ANY, Size::Any, [false; 2], 0b000);

    pub const fn new(prp: Purpose, sz: Size, ee: [bool; 2], ccc: u8) -> Self {
        Self(
            (prp as u16) << 9
                | (sz as u16) << 5
                | (ee[0] as u16) << 4
                | (ee[1] as u16) << 3
                | ccc as u16,
        )
    }
    pub const fn to_byte(&self) -> u8 {
        (self.0 & 0b111) as u8
    }
    pub const fn size(&self) -> Size {
        use std::mem::transmute;
        unsafe { transmute(((self.0 & 0b1111_00000) >> 5) as u8) }
    }
    pub const fn purpose(&self) -> Purpose {
        use std::mem::transmute;
        unsafe { transmute(((self.0 & 0b0001_1110_0000_0000) >> 9) as u8) }
    }
    pub const fn get_ext_bits(&self) -> [bool; 2] {
        [self.0 & 0b10_000 == 0b10_000, self.0 & 0b01_000 == 0b01_000]
    }
    pub fn is_any(&self) -> bool {
        self.0 == Self::__ANY.0
    }
    pub fn is_ctrl_reg(&self) -> bool {
        self.purpose() == Purpose::Ctrl
    }
    pub fn is_dbg_reg(&self) -> bool {
        self.purpose() == Purpose::Dbg
    }
    pub fn is_sgmnt(&self) -> bool {
        self.purpose() == Purpose::Sgmnt
    }
}
