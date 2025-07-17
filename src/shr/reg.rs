// pasm - src/shr/reg.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::size::Size;
use std::str::FromStr;

#[allow(clippy::to_string_trait_impl)]
impl ToString for Register {
    #[rustfmt::skip]
    fn to_string(&self) -> String {
        match *self {
            Self::ES => String::from("es"), Self::SS => String::from("ss"), Self::DS => String::from("ds"),
            Self::CS => String::from("cs"), Self::FS => String::from("fs"), Self::GS => String::from("gs"),
            Self::AL   => String::from("al")  , Self::CL   => String::from("cl")  , Self::DL => String::from("dl"),
            Self::BL   => String::from("bl")  , Self::AH   => String::from("ah")  , Self::CH => String::from("ch"),
            Self::DH   => String::from("dh")  , Self::BH   => String::from("bh")  , Self::R8B => String::from("r8b"),
            Self::R9B  => String::from("r9b") , Self::R10B => String::from("r10b"), Self::R11B => String::from("r11b"),
            Self::R12B => String::from("r12b"), Self::R13B => String::from("r13b"), Self::R14B => String::from("r14b"),
            Self::R15B => String::from("r15b"),
            Self::AX   => String::from("ax")  , Self::CX   => String::from("cx")  , Self::DX   => String::from("dx"),
            Self::BX   => String::from("bx")  , Self::DI   => String::from("di")  , Self::SI   => String::from("si"),
            Self::BP   => String::from("bp")  , Self::SP   => String::from("sp")  , Self::R8W  => String::from("r8w"),
            Self::R9W  => String::from("r9w") , Self::R10W => String::from("r10w"), Self::R11W => String::from("r11w"),
            Self::R12W => String::from("r12w"), Self::R13W => String::from("r13w"), Self::R14W => String::from("r14w"),
            Self::R15W => String::from("r15w"),
            Self::EAX  => String::from("eax") , Self::ECX  => String::from("ecx") , Self::EDX  => String::from("edx") , Self::EBX => String::from("ebx"),
            Self::EDI  => String::from("edi") , Self::ESI  => String::from("esi") , Self::EBP  => String::from("ebp") , Self::ESP => String::from("esp"),
            Self::R8D  => String::from("r8d") , Self::R9D  => String::from("r9d") , Self::R10D => String::from("r10d"), Self::R11D => String::from("r11d"),
            Self::R12D => String::from("r12d"), Self::R13D => String::from("r13d"), Self::R14D => String::from("r14d"), Self::R15D => String::from("r15d"),
            Self::RAX => String::from("rax"), Self::RCX => String::from("rcx"), Self::RDX => String::from("rdx"), Self::RBX => String::from("rbx"),
            Self::RDI => String::from("rdi"), Self::RSI => String::from("rsi"), Self::RBP => String::from("rbp"), Self::RSP => String::from("rsp"),
            Self::R8  => String::from("r8") , Self::R9  => String::from("r9") , Self::R10 => String::from("r10"), Self::R11 => String::from("r11"),
            Self::R12 => String::from("r12"), Self::R13 => String::from("r13"), Self::R14 => String::from("r14"), Self::R15 => String::from("r15"),
            Self::MM0 => String::from("mm0"), Self::MM1 => String::from("mm1"), Self::MM2 => String::from("mm2"), Self::MM3 => String::from("mm3"),
            Self::MM4 => String::from("mm4"), Self::MM5 => String::from("mm5"), Self::MM6 => String::from("mm6"), Self::MM7 => String::from("mm7"),
            Self::K0 => String::from("k0"), Self::K1 => String::from("k1"), Self::K2 => String::from("k2"), Self::K3 => String::from("k3"),
            Self::K4 => String::from("k4"), Self::K5 => String::from("k5"), Self::K6 => String::from("k6"), Self::K7 => String::from("k7"),
            Self::XMM0  => String::from("xmm0") , Self::XMM1  => String::from("xmm1") , Self::XMM2  => String::from("xmm2") , Self::XMM3  => String::from("xmm3"),
            Self::XMM4  => String::from("xmm4") , Self::XMM5  => String::from("xmm5") , Self::XMM6  => String::from("xmm6") , Self::XMM7  => String::from("xmm7"),
            Self::XMM8  => String::from("xmm8") , Self::XMM9  => String::from("xmm9") , Self::XMM10 => String::from("xmm10"), Self::XMM11 => String::from("xmm11"),
            Self::XMM12 => String::from("xmm12"), Self::XMM13 => String::from("xmm13"), Self::XMM14 => String::from("xmm14"), Self::XMM15 => String::from("xmm15"),
            Self::XMM16 => String::from("xmm16"), Self::XMM17 => String::from("xmm17"), Self::XMM18 => String::from("xmm18"), Self::XMM19 => String::from("xmm19"),
            Self::XMM20 => String::from("xmm20"), Self::XMM21 => String::from("xmm21"), Self::XMM22 => String::from("xmm22"), Self::XMM23 => String::from("xmm23"),
            Self::XMM24 => String::from("xmm24"), Self::XMM25 => String::from("xmm25"), Self::XMM26 => String::from("xmm26"), Self::XMM27 => String::from("xmm27"),
            Self::XMM28 => String::from("xmm28"), Self::XMM29 => String::from("xmm29"), Self::XMM30 => String::from("xmm30"), Self::XMM31 => String::from("xmm31"),
            Self::YMM0  => String::from("ymm0") , Self::YMM1  => String::from("ymm1") , Self::YMM2  => String::from("ymm2") , Self::YMM3  => String::from("ymm3"),
            Self::YMM4  => String::from("ymm4") , Self::YMM5  => String::from("ymm5") , Self::YMM6  => String::from("ymm6") , Self::YMM7  => String::from("ymm7"),
            Self::YMM8  => String::from("ymm8") , Self::YMM9  => String::from("ymm9") , Self::YMM10 => String::from("ymm10"), Self::YMM11 => String::from("ymm11"),
            Self::YMM12 => String::from("ymm12"), Self::YMM13 => String::from("ymm13"), Self::YMM14 => String::from("ymm14"), Self::YMM15 => String::from("ymm15"),
            Self::YMM16 => String::from("ymm16"), Self::YMM17 => String::from("ymm17"), Self::YMM18 => String::from("ymm18"), Self::YMM19 => String::from("ymm19"),
            Self::YMM20 => String::from("ymm20"), Self::YMM21 => String::from("ymm21"), Self::YMM22 => String::from("ymm22"), Self::YMM23 => String::from("ymm23"),
            Self::YMM24 => String::from("ymm24"), Self::YMM25 => String::from("ymm25"), Self::YMM26 => String::from("ymm26"), Self::YMM27 => String::from("ymm27"),
            Self::YMM28 => String::from("ymm28"), Self::YMM29 => String::from("ymm29"), Self::YMM30 => String::from("ymm30"), Self::YMM31 => String::from("ymm31"),
            Self::ZMM0  => String::from("zmm0") , Self::ZMM1  => String::from("zmm1") , Self::ZMM2  => String::from("zmm2") , Self::ZMM3  => String::from("zmm3"),
            Self::ZMM4  => String::from("zmm4") , Self::ZMM5  => String::from("zmm5") , Self::ZMM6  => String::from("zmm6") , Self::ZMM7  => String::from("zmm7"),
            Self::ZMM8  => String::from("zmm8") , Self::ZMM9  => String::from("zmm9") , Self::ZMM10 => String::from("zmm10"), Self::ZMM11 => String::from("zmm11"),
            Self::ZMM12 => String::from("zmm12"), Self::ZMM13 => String::from("zmm13"), Self::ZMM14 => String::from("zmm14"), Self::ZMM15 => String::from("zmm15"),
            Self::ZMM16 => String::from("zmm16"), Self::ZMM17 => String::from("zmm17"), Self::ZMM18 => String::from("zmm18"), Self::ZMM19 => String::from("zmm19"),
            Self::ZMM20 => String::from("zmm20"), Self::ZMM21 => String::from("zmm21"), Self::ZMM22 => String::from("zmm22"), Self::ZMM23 => String::from("zmm23"),
            Self::ZMM24 => String::from("zmm24"), Self::ZMM25 => String::from("zmm25"), Self::ZMM26 => String::from("zmm26"), Self::ZMM27 => String::from("zmm27"),
            Self::ZMM28 => String::from("zmm28"), Self::ZMM29 => String::from("zmm29"), Self::ZMM30 => String::from("zmm30"), Self::ZMM31 => String::from("zmm31"),
            Self::CR0 => String::from("cr0"),
            Self::CR1 => String::from("cr1"),
            Self::CR2 => String::from("cr2"),
            Self::CR3 => String::from("cr3"),
            Self::CR4 => String::from("cr4"),
            Self::CR5 => String::from("cr5"),
            Self::CR6 => String::from("cr6"),
            Self::CR7 => String::from("cr7"),
            Self::CR8 => String::from("cr8"),
            Self::CR9 => String::from("cr9"),
            Self::CR10 => String::from("cr10"),
            Self::CR11 => String::from("cr11"),
            Self::CR12 => String::from("cr12"),
            Self::CR13 => String::from("cr13"),
            Self::CR14 => String::from("cr14"),
            Self::CR15 => String::from("cr15"),
            Self::DR0 => String::from("dr0"),
            Self::DR1 => String::from("dr1"),
            Self::DR2 => String::from("dr2"),
            Self::DR3 => String::from("dr3"),
            Self::DR4 => String::from("dr4"),
            Self::DR5 => String::from("dr5"),
            Self::DR6 => String::from("dr6"),
            Self::DR7 => String::from("dr7"),
            Self::DR8 => String::from("dr8"),
            Self::DR9 => String::from("dr9"),
            Self::DR10 => String::from("dr10"),
            Self::DR11 => String::from("dr11"),
            Self::DR12 => String::from("dr12"),
            Self::DR13 => String::from("dr13"),
            Self::DR14 => String::from("dr14"),
            Self::DR15 => String::from("dr15"),
            Self::RIP => String::from("rip"),
            Self::EIP => String::from("eip"),
            Self::IP => String::from("ip"),
            _ => String::new(),
        }
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
    // prepare to check
    pub const fn preptochk(&self) -> u16 {
        self.0 >> 5
    }
}
