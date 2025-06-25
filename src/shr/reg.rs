// pasm - src/shr/reg.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    atype::{AType, ToAType},
    size::Size,
};
use std::{cmp::Ordering, str::FromStr};

#[derive(Debug, Eq, Clone, Copy)]
pub enum Purpose {
    __ANY,
    General,
    IPtr,  // ip/rip/eip
    Dbg,   // drX
    Ctrl,  // crX
    Mmx,   // mmX
    F128,  // xmmX
    F256,  // ymmX
    Sgmnt, // segment registers (cs, ss, ds, es, ...)
}

impl Purpose {
    pub const fn is_any(&self) -> bool {
        *self as u8 == Self::__ANY as u8
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

#[rustfmt::skip]
#[derive(Debug, Clone, Copy)]
pub enum Register {
    // 8-bit general purpose registers
    AL , BL , CL , DL,
    AH , BH , CH , DH,

    // 8-bit extended general purpose registers
    SPL, BPL, SIL, DIL,

    // 16-bit general purpose registers
    AX, BX, CX, DX,
    SP, BP, SI, DI,

    // 32-bit general purpose registers
    EAX, EBX, ECX, EDX,
    ESP, EBP, ESI, EDI,

    // 64-bit general purpose registers
    RAX, RBX, RCX, RDX,
    RSP, RBP, RSI, RDI,

    // 64-bit extended general purpose registers
    R8 , R9 , R10, R11,
    R12, R13, R14, R15,

    // 32-bit extended general purpose registers
    R8D , R9D , R10D, R11D,
    R12D, R13D, R14D, R15D,

    // 16-bit extended general purpose registers
    R8W , R9W , R10W, R11W,
    R12W, R13W, R14W, R15W,

    // 8-bit extended general purpose registers
    R8B , R9B , R10B, R11B,
    R12B, R13B, R14B, R15B,

    // segment registers
    CS, DS, ES, SS, GS, FS,

    // control registers
    CR0 , CR1 , CR2 , CR3,
    CR4 , CR5 , CR6 , CR7,
    CR8 , CR9 , CR10, CR11,
    CR12, CR13, CR14, CR15,

    // debug registers
    DR0 , DR1 , DR2 , DR3,
    DR4 , DR5 , DR6 , DR7,
    DR8 , DR9 , DR10, DR11,
    DR12, DR13, DR14, DR15,

    // Instruction pointers
    RIP, EIP, IP,

    // MMX
    MM0, MM1, MM2, MM3,
    MM4, MM5, MM6, MM7,
    // SSE
    XMM0, XMM1, XMM2, XMM3,
    XMM4, XMM5, XMM6, XMM7,
    
    XMM8 , XMM9 , XMM10, XMM11,
    XMM12, XMM13, XMM14, XMM15,

    // CURRENTLY UNSUPPORTED:
    //  AVX
    YMM0, YMM1, YMM2, YMM3,
    YMM4, YMM5, YMM6, YMM7,
    
    YMM8 , YMM9 , YMM10, YMM11,
    YMM12, YMM13, YMM14, YMM15,

    // Any register
    __ANY,
}

impl PartialEq for Register {
    fn eq(&self, rhs: &Self) -> bool {
        let su8 = *self as u8;
        let ru8 = *rhs as u8;
        if su8 == Register::__ANY as u8 || ru8 == Register::__ANY as u8 {
            true
        } else {
            su8 == ru8
        }
    }
}

#[inline(always)]
fn reg_ie(
    str: &[u8],
    cmp: &'static [u8],
    start: usize,
    end: usize,
    reg: Register,
) -> Result<Register, ()> {
    for idx in start..end {
        let res = str[idx].cmp(&cmp[idx]);
        if res != Ordering::Equal {
            return Err(());
        }
    }
    Ok(reg)
}

impl FromStr for Register {
    type Err = ();
    fn from_str(str: &str) -> Result<Self, <Self as FromStr>::Err> {
        let byte_str = str.as_bytes();
        let str = byte_str;
        match byte_str.len() {
            1 => Err(()),
            2 => match byte_str[1] as char {
                'i' => match byte_str[0] as char {
                    's' => Ok(Register::SI),
                    'd' => Ok(Register::DI),
                    _ => Err(()),
                },
                'l' => match byte_str[0] as char {
                    'a' => Ok(Register::AL),
                    'b' => Ok(Register::BL),
                    'c' => Ok(Register::CL),
                    'd' => Ok(Register::DL),
                    _ => Err(()),
                },
                'x' => match byte_str[0] as char {
                    'a' => Ok(Register::AX),
                    'b' => Ok(Register::BX),
                    'c' => Ok(Register::CX),
                    'd' => Ok(Register::DX),
                    _ => Err(()),
                },
                'h' => match byte_str[0] as char {
                    'a' => Ok(Register::AH),
                    'b' => Ok(Register::BH),
                    'c' => Ok(Register::CH),
                    'd' => Ok(Register::DH),
                    _ => Err(()),
                },
                'p' => match byte_str[0] as char {
                    'i' => Ok(Register::IP),
                    'b' => Ok(Register::BP),
                    's' => Ok(Register::SP),
                    _ => Err(()),
                },
                's' => match byte_str[0] as char {
                    'c' => Ok(Register::CS),
                    'd' => Ok(Register::DS),
                    'e' => Ok(Register::ES),
                    's' => Ok(Register::SS),
                    'f' => Ok(Register::FS),
                    'g' => Ok(Register::GS),
                    _ => Err(()),
                },
                '8' => Ok(Register::R8),
                '9' => Ok(Register::R9),
                _ => Err(()),
            },
            // prev = 2; byte_str.len()
            3 => {
                match byte_str[0] as char {
                    'm' => match byte_str[2] as char {
                        '0' => reg_ie(byte_str, b"mm0", 1, 1, Register::MM0),
                        '1' => reg_ie(byte_str, b"mm1", 1, 1, Register::MM1),
                        '2' => reg_ie(byte_str, b"mm2", 1, 1, Register::MM2),
                        '3' => reg_ie(byte_str, b"mm3", 1, 1, Register::MM3),
                        '4' => reg_ie(byte_str, b"mm4", 1, 1, Register::MM4),
                        '5' => reg_ie(byte_str, b"mm5", 1, 1, Register::MM5),
                        '6' => reg_ie(byte_str, b"mm6", 1, 1, Register::MM6),
                        '7' => reg_ie(byte_str, b"mm7", 1, 1, Register::MM7),
                        _ => Err(()),
                    },
                    'r' => match byte_str[1] as char {
                        'a' => reg_ie(byte_str, b"rax", 2, 2, Register::RAX),
                        'b' => match byte_str[2] as char {
                            'x' => Ok(Register::RBX),
                            'p' => Ok(Register::RBP),
                            _ => Err(()),
                        },
                        'c' => reg_ie(byte_str, b"rcx", 2, 2, Register::RCX),
                        'd' => match byte_str[2] as char {
                            'x' => Ok(Register::RDX),
                            'i' => Ok(Register::RDI),
                            _ => Err(()),
                        },
                        's' => match byte_str[2] as char {
                            'p' => Ok(Register::RSP),
                            'i' => Ok(Register::RSI),
                            _ => Err(()),
                        },
                        '1' => match byte_str[2] as char {
                            '0' => Ok(Register::R10),
                            '1' => Ok(Register::R11),
                            '2' => Ok(Register::R12),
                            '3' => Ok(Register::R13),
                            '4' => Ok(Register::R14),
                            '5' => Ok(Register::R15),
                            _ => Err(()),
                        },
                        '8' => match byte_str[2] as char {
                            'b' => Ok(Register::R8B),
                            'w' => Ok(Register::R8W),
                            'd' => Ok(Register::R8D),
                            _ => Err(()),
                        },
                        '9' => match byte_str[2] as char {
                            'b' => Ok(Register::R9B),
                            'w' => Ok(Register::R9W),
                            'd' => Ok(Register::R9D),
                            _ => Err(()),
                        },
                        'i' => reg_ie(byte_str, b"rip", 2, 2, Register::RIP),
                        _ => Err(()),
                    },
                    // prev = 'r'; byte_str[0]
                    'e' => match byte_str[1] as char {
                        'a' => reg_ie(byte_str, b"eax", 2, 2, Register::EAX),
                        'b' => match byte_str[2] as char {
                            'p' => Ok(Register::EBP),
                            'x' => Ok(Register::EBX),
                            _ => Err(()),
                        },
                        'c' => reg_ie(byte_str, b"ecx", 2, 2, Register::ECX),
                        'd' => match byte_str[2] as char {
                            'i' => Ok(Register::EDI),
                            'x' => Ok(Register::EDX),
                            _ => Err(()),
                        },
                        's' => match byte_str[2] as char {
                            'i' => Ok(Register::ESI),
                            'p' => Ok(Register::ESP),
                            _ => Err(()),
                        },
                        'i' => reg_ie(str, b"eip", 2, 2, Register::EIP),
                        _ => Err(()),
                    },
                    // prev = 'e'; byte_str[0]
                    's' => match byte_str[1] as char {
                        'p' => reg_ie(str, b"spl", 2, 2, Register::SPL),
                        'i' => reg_ie(str, b"sil", 2, 2, Register::SIL),
                        _ => Err(()),
                    },
                    'b' => reg_ie(str, b"bpl", 1, 2, Register::BPL),
                    'c' => match byte_str[2] as char {
                        '0' => reg_ie(str, b"cr0", 1, 1, Register::CR0),
                        '1' => reg_ie(str, b"cr1", 1, 1, Register::CR1),
                        '2' => reg_ie(str, b"cr2", 1, 1, Register::CR2),
                        '3' => reg_ie(str, b"cr3", 1, 1, Register::CR3),
                        '4' => reg_ie(str, b"cr4", 1, 1, Register::CR4),
                        '5' => reg_ie(str, b"cr5", 1, 1, Register::CR5),
                        '6' => reg_ie(str, b"cr6", 1, 1, Register::CR6),
                        '7' => reg_ie(str, b"cr7", 1, 1, Register::CR7),
                        '8' => reg_ie(str, b"cr8", 1, 1, Register::CR8),
                        '9' => reg_ie(str, b"cr9", 1, 1, Register::CR9),
                        _ => Err(()),
                    },
                    'd' => match byte_str[2] as char {
                        'i' => reg_ie(str, b"dil", 1, 1, Register::DIL),
                        '0' => reg_ie(str, b"dr0", 1, 1, Register::DR0),
                        '1' => reg_ie(str, b"dr1", 1, 1, Register::DR1),
                        '2' => reg_ie(str, b"dr2", 1, 1, Register::DR2),
                        '3' => reg_ie(str, b"dr3", 1, 1, Register::DR3),
                        '4' => reg_ie(str, b"dr4", 1, 1, Register::DR4),
                        '5' => reg_ie(str, b"dr5", 1, 1, Register::DR5),
                        '6' => reg_ie(str, b"dr6", 1, 1, Register::DR6),
                        '7' => reg_ie(str, b"dr7", 1, 1, Register::DR7),
                        '8' => reg_ie(str, b"dr8", 1, 1, Register::DR8),
                        '9' => reg_ie(str, b"dr9", 1, 1, Register::DR9),
                        _ => Err(()),
                    },
                    _ => Err(()),
                }
            }
            // prev = 3; byte_str.len()
            4 => match byte_str[0] as char {
                'c' => match byte_str[3] as char {
                    '0' => reg_ie(str, b"cr10", 1, 2, Register::CR10),
                    '1' => reg_ie(str, b"cr11", 1, 2, Register::CR11),
                    '2' => reg_ie(str, b"cr12", 1, 2, Register::CR12),
                    '3' => reg_ie(str, b"cr13", 1, 2, Register::CR13),
                    '4' => reg_ie(str, b"cr14", 1, 2, Register::CR14),
                    '5' => reg_ie(str, b"cr15", 1, 2, Register::CR15),
                    _ => Err(()),
                },
                'd' => match byte_str[3] as char {
                    '0' => reg_ie(str, b"dr10", 1, 2, Register::DR10),
                    '1' => reg_ie(str, b"dr11", 1, 2, Register::DR11),
                    '2' => reg_ie(str, b"dr12", 1, 2, Register::DR12),
                    '3' => reg_ie(str, b"dr13", 1, 2, Register::DR13),
                    '4' => reg_ie(str, b"dr14", 1, 2, Register::DR14),
                    '5' => reg_ie(str, b"dr15", 1, 2, Register::DR15),
                    _ => Err(()),
                },
                'r' => match byte_str[2] as char {
                    '0' => match byte_str[3] as char {
                        'd' => reg_ie(str, b"r10d", 1, 1, Register::R10D),
                        'b' => reg_ie(str, b"r10b", 1, 1, Register::R10B),
                        'w' => reg_ie(str, b"r10w", 1, 1, Register::R10W),
                        _ => Err(()),
                    },
                    '1' => match byte_str[3] as char {
                        'd' => reg_ie(str, b"r11d", 1, 1, Register::R11D),
                        'b' => reg_ie(str, b"r11b", 1, 1, Register::R11B),
                        'w' => reg_ie(str, b"r11w", 1, 1, Register::R11W),
                        _ => Err(()),
                    },
                    '2' => match byte_str[3] as char {
                        'd' => reg_ie(str, b"r12d", 1, 1, Register::R12D),
                        'b' => reg_ie(str, b"r12b", 1, 1, Register::R12B),
                        'w' => reg_ie(str, b"r12w", 1, 1, Register::R12W),
                        _ => Err(()),
                    },
                    '3' => match byte_str[3] as char {
                        'd' => reg_ie(str, b"r13d", 1, 1, Register::R13D),
                        'b' => reg_ie(str, b"r13b", 1, 1, Register::R13B),
                        'w' => reg_ie(str, b"r13w", 1, 1, Register::R13W),
                        _ => Err(()),
                    },
                    '4' => match byte_str[3] as char {
                        'b' => reg_ie(str, b"r14b", 1, 1, Register::R14B),
                        'w' => reg_ie(str, b"r14w", 1, 1, Register::R14W),
                        'd' => reg_ie(str, b"r14d", 1, 1, Register::R14D),
                        _ => Err(()),
                    },
                    '5' => match byte_str[3] as char {
                        'b' => reg_ie(str, b"r15b", 1, 1, Register::R15B),
                        'w' => reg_ie(str, b"r15w", 1, 1, Register::R15W),
                        'd' => reg_ie(str, b"r15d", 1, 1, Register::R15D),
                        _ => Err(()),
                    },
                    _ => Err(()),
                },
                'x' => match byte_str[3] as char {
                    '0' => reg_ie(str, b"xmm0", 1, 2, Register::XMM0),
                    '1' => reg_ie(str, b"xmm1", 1, 2, Register::XMM1),
                    '2' => reg_ie(str, b"xmm2", 1, 2, Register::XMM2),
                    '3' => reg_ie(str, b"xmm3", 1, 2, Register::XMM3),
                    '4' => reg_ie(str, b"xmm4", 1, 2, Register::XMM4),
                    '5' => reg_ie(str, b"xmm5", 1, 2, Register::XMM5),
                    '6' => reg_ie(str, b"xmm6", 1, 2, Register::XMM6),
                    '7' => reg_ie(str, b"xmm7", 1, 2, Register::XMM7),
                    '8' => reg_ie(str, b"xmm8", 1, 2, Register::XMM8),
                    '9' => reg_ie(str, b"xmm9", 1, 2, Register::XMM9),
                    _ => Err(()),
                },
                'y' => match byte_str[3] as char {
                    '0' => reg_ie(str, b"ymm0", 1, 2, Register::YMM0),
                    '1' => reg_ie(str, b"ymm1", 1, 2, Register::YMM1),
                    '2' => reg_ie(str, b"ymm2", 1, 2, Register::YMM2),
                    '3' => reg_ie(str, b"ymm3", 1, 2, Register::YMM3),
                    '4' => reg_ie(str, b"ymm4", 1, 2, Register::YMM4),
                    '5' => reg_ie(str, b"ymm5", 1, 2, Register::YMM5),
                    '6' => reg_ie(str, b"ymm6", 1, 2, Register::YMM6),
                    '7' => reg_ie(str, b"ymm7", 1, 2, Register::YMM7),
                    '8' => reg_ie(str, b"ymm8", 1, 2, Register::YMM8),
                    '9' => reg_ie(str, b"ymm9", 1, 2, Register::YMM9),
                    _ => Err(()),
                },
                _ => Err(()),
            },
            5 => match byte_str[0] as char {
                'x' => match byte_str[4] as char {
                    '0' => reg_ie(str, b"xmm10", 1, 3, Register::XMM10),
                    '1' => reg_ie(str, b"xmm11", 1, 3, Register::XMM11),
                    '2' => reg_ie(str, b"xmm12", 1, 3, Register::XMM12),
                    '3' => reg_ie(str, b"xmm13", 1, 3, Register::XMM13),
                    '4' => reg_ie(str, b"xmm14", 1, 3, Register::XMM14),
                    '5' => reg_ie(str, b"xmm15", 1, 3, Register::XMM15),
                    _ => Err(()),
                },
                'y' => match byte_str[4] as char {
                    '0' => reg_ie(str, b"ymm10", 1, 3, Register::YMM10),
                    '1' => reg_ie(str, b"ymm11", 1, 3, Register::YMM11),
                    '2' => reg_ie(str, b"ymm12", 1, 3, Register::YMM12),
                    '3' => reg_ie(str, b"ymm13", 1, 3, Register::YMM13),
                    '4' => reg_ie(str, b"ymm14", 1, 3, Register::YMM14),
                    '5' => reg_ie(str, b"ymm15", 1, 3, Register::YMM15),
                    _ => Err(()),
                },
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

impl ToAType for Register {
    fn atype(&self) -> AType {
        AType::Register(self.purpose(), self.size())
    }
}

impl Register {
    #[inline]
    pub fn is_dbg_reg(&self) -> bool {
        self.purpose() == Purpose::Dbg
    }
    #[inline]
    pub fn is_ctrl_reg(&self) -> bool {
        self.purpose() == Purpose::Ctrl
    }
    #[rustfmt::skip]
    pub const fn size(&self) -> Size {
        match self {
            Self::__ANY => Size::Unknown,

            Self::AL  | Self::BL | Self::CL  | Self::DL |
            Self::AH  | Self::BH | Self::CH  | Self::DH |
            Self::SPL | Self::BPL| Self::SIL | Self::DIL|
            Self::R8B |Self::R9B |Self::R10B |Self::R11B|
            Self::R12B|Self::R13B| Self::R14B| Self::R15B => Size::Byte,

            Self::CS  | Self::DS | Self::ES | Self::SS | Self::FS | Self::GS  |
            Self::AX  | Self::BX | Self::CX | Self::DX | Self::SP | Self::BP  |
            Self::SI  | Self::DI | Self::IP | Self::R8W| Self::R9W| Self::R10W|
            Self::R11W|Self::R12W|Self::R13W|Self::R14W| Self::R15W => Size::Word,

            Self::EAX | Self::EBX | Self::ECX | Self::EDX |
            Self::ESP | Self::EBP | Self::ESI | Self::EDI | Self::EIP|
            Self::CR0 | Self::CR1 | Self::CR2 | Self::CR3 |
            Self::CR4 | Self::CR5 | Self::CR6 | Self::CR7 |
            Self::CR8 | Self::CR9 | Self::CR10| Self::CR11|
            Self::CR12|Self::CR13 | Self::CR14| Self::CR15|
            Self::DR0 | Self::DR1 | Self::DR2 | Self::DR3 |
            Self::DR4 | Self::DR5 | Self::DR6 | Self::DR7 |
            Self::DR8 | Self::DR9 | Self::DR10| Self::DR11|
            Self::DR12| Self::DR13| Self::DR14| Self::DR15|
            Self::R8D | Self::R9D | Self::R10D| Self::R11D|
            Self::R12D| Self::R13D| Self::R14D| Self::R15D => Size::Dword,

            Self::RAX | Self::RBX | Self::RCX | Self::RDX |
            Self::RSP | Self::RBP | Self::RSI | Self::RDI |
            Self::R8  | Self::R9  | Self::R10 | Self::R11 |
            Self::RIP | Self::R12 | Self::R13 | Self::R14 |
            Self::MM0 | Self::MM1 | Self::MM2 | Self::MM3 |
            Self::MM4 | Self::MM5 | Self::MM6 | Self::MM7 |
            Self::R15 => Size::Qword,

            Self::XMM0 | Self::XMM1 | Self::XMM2 | Self::XMM3 |
            Self::XMM4 | Self::XMM5 | Self::XMM6 | Self::XMM7 |
            Self::XMM8 | Self::XMM9 | Self::XMM10| Self::XMM11|
            Self::XMM12| Self::XMM13| Self::XMM14| Self::XMM15 => Size::Xword,

            Self::YMM0 | Self::YMM1 | Self::YMM2 | Self::YMM3 |
            Self::YMM4 | Self::YMM5 | Self::YMM6 | Self::YMM7 |
            Self::YMM8 | Self::YMM9 | Self::YMM10| Self::YMM11|
            Self::YMM12| Self::YMM13| Self::YMM14| Self::YMM15 => Size::Yword,
        }
    }
    #[rustfmt::skip]
    pub const fn needs_rex(&self) -> bool {
        matches!(
            self,
            Self::R8   | Self::R9   | Self::R10  | Self::R11  |
            Self::R12  | Self::R13  | Self::R14  | Self::R15  |
            Self::R8B  | Self::R9B  | Self::R10B | Self::R11B |
            Self::R12B | Self::R13B | Self::R14B | Self::R15B |
            Self::R8W  | Self::R9W  | Self::R10W | Self::R11W |
            Self::R12W | Self::R13W | Self::R14W | Self::R15W |
            Self::R8D  | Self::R9D  | Self::R10D | Self::R11D |
            Self::R12D | Self::R13D | Self::R14D | Self::R15D |
            Self::XMM8 | Self::XMM9 | Self::XMM10| Self::XMM11|
            Self::XMM12| Self::XMM13| Self::XMM14| Self::XMM15|
            Self::YMM8 | Self::YMM9 | Self::YMM10| Self::YMM11|
            Self::SIL  | Self::DIL  | Self::BPL  | Self::SPL  |
            Self::CR8  | Self::CR9  | Self::CR10 | Self::CR11 |
            Self::CR12 | Self::CR13 | Self::CR14 | Self::CR15 |
            Self::DR8  | Self::DR9  | Self::DR10 | Self::DR11 |
            Self::DR12 | Self::DR13 | Self::DR14 | Self::DR15 |
            Self::YMM12| Self::YMM13| Self::YMM14| Self::YMM15
        )
    }
    #[rustfmt::skip]
    pub const fn to_byte(&self) -> u8 {
        match &self {
            Self::__ANY => 0b000,
            Self::ES   | Self::MM0 |
            Self::R8   | Self::R8B | Self::R8W  | Self::R8D   |
            Self::XMM8 | Self::YMM8| Self::AL   | Self::AX    |
            Self::EAX  | Self::CR0 | Self::CR8  | Self::DR0   |
            Self::DR8  | Self::RAX | Self::XMM0 | Self::YMM0   => 0b000,

            Self::CS   | Self::MM1 |
            Self::R9   | Self::R9B | Self::R9W  | Self::R9D   |
            Self::CL   | Self::CX  | Self::ECX  | Self::RCX   |
            Self::XMM1 | Self::YMM1| Self::XMM9 | Self::CR1   |
            Self::YMM9 | Self::CR9 | Self::DR1  | Self::DR9    => 0b001,

            Self::SS   | Self::MM2 |
            Self::R10  | Self::R10B| Self::R10W | Self::R10D  |
            Self::DL   | Self::DX  | Self::EDX  | Self::XMM2  |
            Self::RDX  | Self::CR2 | Self::CR10 | Self::DR2   |
            Self::DR10 | Self::YMM2| Self::XMM10| Self::YMM10  => 0b010,

            Self::DS   | Self::MM3 |
            Self::R11  | Self::R11B| Self::R11W | Self::R11D |
            Self::BL   | Self::BX  | Self::EBX  | Self::XMM3 |
            Self::RBX  | Self::CR3 | Self::CR11 | Self::DR3  |
            Self::DR11 | Self::YMM3| Self::XMM11| Self::YMM11 => 0b011,

            Self::FS  | Self::MM4  |
            Self::R12 | Self::R12B | Self::R12W | Self::R12D |
            Self::AH  | Self::SP   | Self::ESP  | Self::XMM4 |
            Self::SPL | Self::RSP  | Self::CR4  | Self::CR12 |
            Self::DR4 | Self::DR12 | Self::YMM4 | Self::XMM12|
            Self::YMM12                                       => 0b100,

            Self::GS  | Self::MM5  |
            Self::R13 | Self::R13B | Self::R13W | Self::R13D |
            Self::CH  | Self::BP   | Self::EBP  | Self::XMM5 |
            Self::BPL | Self::RBP  | Self::CR5  | Self::CR13 |
            Self::DR5 | Self::DR13 | Self::YMM5 | Self::XMM13|
            Self::YMM13                                       => 0b101,

            Self::R14   | Self::R14B | Self::R14W | Self::R14D |
            Self::DH    | Self::SI   | Self::ESI  | Self::XMM6 |
            Self::SIL   | Self::RSI  | Self::CR6  | Self::CR14 |
            Self::DR6   | Self::DR14 | Self::YMM6 | Self::XMM14|
            Self::YMM14 | Self::MM6                 => 0b110,

            Self::R15   | Self::R15B | Self::R15W | Self::R15D |
            Self::BH    | Self::DI   | Self::EDI  | Self::XMM7 |
            Self::DIL   | Self::RDI  | Self::CR7  | Self::CR15 |
            Self::DR7   | Self::DR15 | Self::YMM7 | Self::XMM15|
            Self::YMM15 | Self::MM7                  => 0b111,

            Self::IP | Self::EIP | Self::RIP => 0b000
        }
    }
    pub fn is_sgmnt(&self) -> bool {
        self.purpose() == Purpose::Sgmnt
    }
    #[rustfmt::skip]
    pub const fn purpose(&self) -> Purpose{
        match self {
            Self::__ANY => Purpose::__ANY,

            Self::AX  | Self::AL  | Self::EAX  | Self::RAX |
            Self::DX  | Self::DL  | Self::EDX  | Self::RDX |
            Self::CX  | Self::CL  | Self::ECX  | Self::RCX |
            Self::BX  | Self::BL  | Self::EBX  | Self::RBX |
            Self::SP  | Self::SPL | Self::ESP  | Self::RSP |
            Self::BP  | Self::BPL | Self::EBP  | Self::RBP |
            Self::SI  | Self::SIL | Self::ESI  | Self::RSI |
            Self::DI  | Self::DIL | Self::EDI  | Self::RDI |
            Self::AH  | Self::DH  | Self::CH   | Self::BH  |
            Self::R8B | Self::R8W | Self::R8D  | Self::R8  |
            Self::R9B | Self::R9W | Self::R9D  | Self::R9  |
            Self::R10B| Self::R10W| Self::R10D | Self::R10 |
            Self::R11B| Self::R11W| Self::R11D | Self::R11 |
            Self::R12B| Self::R12W| Self::R12D | Self::R12 |
            Self::R13B| Self::R13W| Self::R13D | Self::R13 |
            Self::R14B| Self::R14W| Self::R14D | Self::R14 |
            Self::R15B| Self::R15W| Self::R15D | Self::R15 => Purpose::General,

            Self::CS  | Self::DS  | Self::ES   | Self::SS  |
            Self::FS  | Self::GS  => Purpose::Sgmnt,

            Self::CR0 | Self::CR1 | Self::CR2  | Self::CR3  |
            Self::CR4 | Self::CR5 | Self::CR6  | Self::CR7  |
            Self::CR8 | Self::CR9 | Self::CR10 | Self::CR11 |
            Self::CR12| Self::CR13| Self::CR14 | Self::CR15 => Purpose::Ctrl,

            Self::DR0 | Self::DR1 | Self::DR2  | Self::DR3  |
            Self::DR4 | Self::DR5 | Self::DR6  | Self::DR7  |
            Self::DR8 | Self::DR9 | Self::DR10 | Self::DR11 |
            Self::DR12| Self::DR13| Self::DR14 | Self::DR15 => Purpose::Dbg,

            Self::IP  | Self::EIP | Self::RIP => Purpose::IPtr,

            Self::XMM0 | Self::XMM1 | Self::XMM2 | Self::XMM3  |
            Self::XMM4 | Self::XMM5 | Self::XMM6 | Self::XMM7  |
            Self::XMM8 | Self::XMM9 | Self::XMM10| Self::XMM11 |
            Self::XMM12| Self::XMM13| Self::XMM14| Self::XMM15 => Purpose::F128,

            Self::YMM0 | Self::YMM1 | Self::YMM2 | Self::YMM3  |
            Self::YMM4 | Self::YMM5 | Self::YMM6 | Self::YMM7  |
            Self::YMM8 | Self::YMM9 | Self::YMM10| Self::YMM11 |
            Self::YMM12| Self::YMM13| Self::YMM14| Self::YMM15 => Purpose::F256,

            Self::MM0 | Self::MM1 | Self::MM2 | Self::MM3  |
            Self::MM4 | Self::MM5 | Self::MM6 | Self::MM7  => Purpose::Mmx,
        }
    }
    // For mksek(), se() and de():
    // 0b000_XX_YYYY_ZZZZ_AAA
    // X0 - reserved
    // X1 - extended register
    //
    // YYYY - size
    // ZZZZ - purpose
    // AAA  - register code
    pub const fn mksek(ext: bool, sz: u16, prp: u16, cd: u16) -> u16 {
        (ext as u16) << 11 | sz << 7 | prp << 3 | cd
    }
    pub const fn se(&self) -> u16 {
        let mut tret = 0;

        tret |= self.to_byte() as u16;

        tret |= (self.purpose() as u16) << 3;
        tret |= (self.size() as u16) << 7;
        tret |= (self.needs_rex() as u16) << 11;

        tret
    }
    pub const fn is_any(&self) -> bool {
        *self as u8 == Self::__ANY as u8
    }
    pub const fn de(data: u16) -> Self {
        let code = data & 0b111;
        let purp = data & 0b1111_000;
        let purp = unsafe { std::mem::transmute::<u8, Purpose>((purp >> 3) as u8) };
        let size = unsafe { std::mem::transmute::<u8, Size>(((data & 0b1111 << 7) >> 7) as u8) };
        let extr = (data & 0b01 << 11) >> 11;

        match purp {
            Purpose::__ANY => Self::__ANY,
            Purpose::IPtr => match size {
                Size::Word => Register::IP,
                Size::Dword => Register::EIP,
                Size::Qword => Register::RIP,
                _ => Register::__ANY,
            },
            Purpose::Sgmnt => match (extr & 0b01 == 0b01, code) {
                (false, 0b000) => Self::ES,
                (false, 0b001) => Self::CS,
                (false, 0b010) => Self::SS,
                (false, 0b011) => Self::DS,
                (false, 0b100) => Self::FS,
                (false, 0b101) => Self::GS,
                _ => Self::__ANY,
            },
            Purpose::Mmx => match (extr & 0b01 == 0b01, code) {
                (false, 0b000) => Self::MM0,
                (false, 0b001) => Self::MM1,
                (false, 0b010) => Self::MM2,
                (false, 0b011) => Self::MM3,
                (false, 0b100) => Self::MM4,
                (false, 0b101) => Self::MM5,
                (false, 0b110) => Self::MM6,
                (false, 0b111) => Self::MM7,
                _ => Self::__ANY,
            },
            Purpose::F256 => match (extr & 0b01 == 0b01, code) {
                (false, 0b000) => Self::YMM0,
                (false, 0b001) => Self::YMM1,
                (false, 0b010) => Self::YMM2,
                (false, 0b011) => Self::YMM3,
                (false, 0b100) => Self::YMM4,
                (false, 0b101) => Self::YMM5,
                (false, 0b110) => Self::YMM6,
                (false, 0b111) => Self::YMM7,
                (true, 0b000) => Self::YMM8,
                (true, 0b001) => Self::YMM9,
                (true, 0b010) => Self::YMM10,
                (true, 0b011) => Self::YMM11,
                (true, 0b100) => Self::YMM12,
                (true, 0b101) => Self::YMM13,
                (true, 0b110) => Self::YMM14,
                (true, 0b111) => Self::YMM15,
                _ => Self::__ANY,
            },
            Purpose::F128 => match (extr & 0b01 == 0b01, code) {
                (false, 0b000) => Self::XMM0,
                (false, 0b001) => Self::XMM1,
                (false, 0b010) => Self::XMM2,
                (false, 0b011) => Self::XMM3,
                (false, 0b100) => Self::XMM4,
                (false, 0b101) => Self::XMM5,
                (false, 0b110) => Self::XMM6,
                (false, 0b111) => Self::XMM7,
                (true, 0b000) => Self::XMM8,
                (true, 0b001) => Self::XMM9,
                (true, 0b010) => Self::XMM10,
                (true, 0b011) => Self::XMM11,
                (true, 0b100) => Self::XMM12,
                (true, 0b101) => Self::XMM13,
                (true, 0b110) => Self::XMM14,
                (true, 0b111) => Self::XMM15,
                _ => Self::__ANY,
            },
            Purpose::Ctrl => match (extr & 0b01 == 0b01, code) {
                (false, 0b000) => Self::CR0,
                (false, 0b001) => Self::CR1,
                (false, 0b010) => Self::CR2,
                (false, 0b011) => Self::CR3,
                (false, 0b100) => Self::CR4,
                (false, 0b101) => Self::CR5,
                (false, 0b110) => Self::CR6,
                (false, 0b111) => Self::CR7,
                (true, 0b000) => Self::CR8,
                (true, 0b001) => Self::CR9,
                (true, 0b010) => Self::CR10,
                (true, 0b011) => Self::CR11,
                (true, 0b100) => Self::CR12,
                (true, 0b101) => Self::CR13,
                (true, 0b110) => Self::CR14,
                (true, 0b111) => Self::CR15,
                _ => Self::__ANY,
            },
            Purpose::Dbg => match (extr & 0b01 == 0b01, code) {
                (false, 0b000) => Self::DR0,
                (false, 0b001) => Self::DR1,
                (false, 0b010) => Self::DR2,
                (false, 0b011) => Self::DR3,
                (false, 0b100) => Self::DR4,
                (false, 0b101) => Self::DR5,
                (false, 0b110) => Self::DR6,
                (false, 0b111) => Self::DR7,
                (true, 0b000) => Self::DR8,
                (true, 0b001) => Self::DR9,
                (true, 0b010) => Self::DR10,
                (true, 0b011) => Self::DR11,
                (true, 0b100) => Self::DR12,
                (true, 0b101) => Self::DR13,
                (true, 0b110) => Self::DR14,
                (true, 0b111) => Self::DR15,
                _ => Self::__ANY,
            },
            Purpose::General => match size {
                Size::Byte => match (extr & 0b01 == 0b01, code) {
                    (false, 0b000) => Self::AL,
                    (false, 0b001) => Self::CL,
                    (false, 0b010) => Self::DL,
                    (false, 0b011) => Self::BL,
                    (false, 0b100) => Self::AH,
                    (false, 0b101) => Self::CH,
                    (false, 0b110) => Self::DH,
                    (false, 0b111) => Self::BH,
                    (true, 0b000) => Self::R8B,
                    (true, 0b001) => Self::R9B,
                    (true, 0b010) => Self::R10B,
                    (true, 0b011) => Self::R11B,
                    (true, 0b100) => Self::R12B,
                    (true, 0b101) => Self::R13B,
                    (true, 0b110) => Self::R14B,
                    (true, 0b111) => Self::R15B,
                    _ => Self::__ANY,
                },
                Size::Word => match (extr & 0b01 == 0b01, code) {
                    (false, 0b000) => Self::AX,
                    (false, 0b001) => Self::CX,
                    (false, 0b010) => Self::DX,
                    (false, 0b011) => Self::BX,
                    (false, 0b100) => Self::SP,
                    (false, 0b101) => Self::BP,
                    (false, 0b110) => Self::SI,
                    (false, 0b111) => Self::DI,
                    (true, 0b000) => Self::R8W,
                    (true, 0b001) => Self::R9W,
                    (true, 0b010) => Self::R10W,
                    (true, 0b011) => Self::R11W,
                    (true, 0b100) => Self::R12W,
                    (true, 0b101) => Self::R13W,
                    (true, 0b110) => Self::R14W,
                    (true, 0b111) => Self::R15W,
                    _ => Self::__ANY,
                },
                Size::Dword => match (extr & 0b01 == 0b01, code) {
                    (false, 0b000) => Self::EAX,
                    (false, 0b001) => Self::ECX,
                    (false, 0b010) => Self::EDX,
                    (false, 0b011) => Self::EBX,
                    (false, 0b100) => Self::ESP,
                    (false, 0b101) => Self::EBP,
                    (false, 0b110) => Self::ESI,
                    (false, 0b111) => Self::EDI,
                    (true, 0b000) => Self::R8D,
                    (true, 0b001) => Self::R9D,
                    (true, 0b010) => Self::R10D,
                    (true, 0b011) => Self::R11D,
                    (true, 0b100) => Self::R12D,
                    (true, 0b101) => Self::R13D,
                    (true, 0b110) => Self::R14D,
                    (true, 0b111) => Self::R15D,
                    _ => Self::__ANY,
                },
                Size::Qword => match (extr & 0b01 == 0b01, code) {
                    (false, 0b000) => Self::RAX,
                    (false, 0b001) => Self::RCX,
                    (false, 0b010) => Self::RDX,
                    (false, 0b011) => Self::RBX,
                    (false, 0b100) => Self::RSP,
                    (false, 0b101) => Self::RBP,
                    (false, 0b110) => Self::RSI,
                    (false, 0b111) => Self::RDI,
                    (true, 0b000) => Self::R8,
                    (true, 0b001) => Self::R9,
                    (true, 0b010) => Self::R10,
                    (true, 0b011) => Self::R11,
                    (true, 0b100) => Self::R12,
                    (true, 0b101) => Self::R13,
                    (true, 0b110) => Self::R14,
                    (true, 0b111) => Self::R15,
                    _ => Self::__ANY,
                },
                _ => Self::__ANY,
            },
        }
    }
}

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
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Register {
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}
