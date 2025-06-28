// pasm - src/shr/reg.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    atype::{AType, ToAType},
    size::Size,
};
use std::str::FromStr;

#[derive(Debug, Eq, Clone, Copy)]
pub enum Purpose {
    __ANY,
    General,
    IPtr,  // ip/rip/eip
    Dbg,   // drX
    Ctrl,  // crX
    Mask,  // k0-7
    Mmx,   // mmX
    F128,  // xmmX
    F256,  // ymmX
    F512,  // zmmX
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

    //  AVX
    YMM0, YMM1, YMM2, YMM3,
    YMM4, YMM5, YMM6, YMM7,
    
    YMM8 , YMM9 , YMM10, YMM11,
    YMM12, YMM13, YMM14, YMM15,
    
    // AVX512
    XMM16, XMM17, XMM18, XMM19,
    XMM20, XMM21, XMM22, XMM23,
    XMM24, XMM25, XMM26, XMM27,
    XMM28, XMM29, XMM30, XMM31,
    
    YMM16, YMM17, YMM18, YMM19,
    YMM20, YMM21, YMM22, YMM23,
    YMM24, YMM25, YMM26, YMM27,
    YMM28, YMM29, YMM30, YMM31,

    ZMM0 , ZMM1 , ZMM2 , ZMM3 ,
    ZMM4 , ZMM5 , ZMM6 , ZMM7 ,
    ZMM8 , ZMM9 , ZMM10, ZMM11,
    ZMM12, ZMM13, ZMM14, ZMM15,
    ZMM16, ZMM17, ZMM18, ZMM19,
    ZMM20, ZMM21, ZMM22, ZMM23,
    ZMM24, ZMM25, ZMM26, ZMM27,
    ZMM28, ZMM29, ZMM30, ZMM31,

    // AVX-512 masks
    K0, K1, K2, K3,
    K4, K5, K6, K7,

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

    pub const fn get_ext_bits(&self) -> [bool; 2] {
        use Register::*;
        match self {
            SIL | DIL | BPL | SPL | CR8 | CR9 | CR10 | CR11 | CR12 | CR13 | CR14 | CR15 | DR8
            | DR9 | DR10 | DR11 | DR12 | DR13 | DR14 | DR15 | R8B | R9B | R10B | R11B | R12B
            | R13B | R14B | R15B | R8W | R9W | R10W | R11W | R12W | R13W | R14W | R15W | R8D
            | R9D | R10D | R11D | R12D | R13D | R14D | R15D | R8 | R9 | R10 | R11 | R12 | R13
            | R14 | R15 => [false, true],

            XMM8 | XMM9 | XMM10 | XMM11 | XMM12 | XMM13 | XMM14 | XMM15 | YMM8 | YMM9 | YMM10
            | YMM11 | YMM12 | YMM13 | YMM14 | YMM15 | ZMM8 | ZMM9 | ZMM10 | ZMM11 | ZMM12
            | ZMM13 | ZMM14 | ZMM15 => [false, true],

            XMM16 | XMM17 | XMM18 | XMM19 | XMM20 | XMM21 | XMM22 | XMM23 | YMM16 | YMM17
            | YMM18 | YMM19 | YMM20 | YMM21 | YMM22 | YMM23 | ZMM16 | ZMM17 | ZMM18 | ZMM19
            | ZMM20 | ZMM21 | ZMM22 | ZMM23 => [true, false],

            XMM24 | XMM25 | XMM26 | XMM27 | XMM28 | XMM29 | XMM30 | XMM31 | YMM24 | YMM25
            | YMM26 | YMM27 | YMM28 | YMM29 | YMM30 | YMM31 | ZMM24 | ZMM25 | ZMM26 | ZMM27
            | ZMM28 | ZMM29 | ZMM30 | ZMM31 => [true, true],

            _ => [false; 2],
        }
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


            Self::K0  | Self::K1  | Self::K2  | Self::K3  |
            Self::K4  | Self::K5  | Self::K6  | Self::K7  |
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
            Self::XMM12| Self::XMM13| Self::XMM14| Self::XMM15|
            Self::XMM16| Self::XMM17| Self::XMM18| Self::XMM19|
            Self::XMM20| Self::XMM21| Self::XMM22| Self::XMM23|
            Self::XMM24| Self::XMM25| Self::XMM26| Self::XMM27|
            Self::XMM28| Self::XMM29| Self::XMM30| Self::XMM31 => Size::Xword,

            Self::YMM0 | Self::YMM1 | Self::YMM2 | Self::YMM3 |
            Self::YMM4 | Self::YMM5 | Self::YMM6 | Self::YMM7 |
            Self::YMM8 | Self::YMM9 | Self::YMM10| Self::YMM11|
            Self::YMM12| Self::YMM13| Self::YMM14| Self::YMM15|
            Self::YMM16| Self::YMM17| Self::YMM18| Self::YMM19|
            Self::YMM20| Self::YMM21| Self::YMM22| Self::YMM23|
            Self::YMM24| Self::YMM25| Self::YMM26| Self::YMM27|
            Self::YMM28| Self::YMM29| Self::YMM30| Self::YMM31 => Size::Yword,
            Self::ZMM0 | Self::ZMM1 | Self::ZMM2 | Self::ZMM3 |
            Self::ZMM4 | Self::ZMM5 | Self::ZMM6 | Self::ZMM7 |
            Self::ZMM8 | Self::ZMM9 | Self::ZMM10| Self::ZMM11|
            Self::ZMM12| Self::ZMM13| Self::ZMM14| Self::ZMM15|
            Self::ZMM16| Self::ZMM17| Self::ZMM18| Self::ZMM19|
            Self::ZMM20| Self::ZMM21| Self::ZMM22| Self::ZMM23|
            Self::ZMM24| Self::ZMM25| Self::ZMM26| Self::ZMM27|
            Self::ZMM28| Self::ZMM29| Self::ZMM30| Self::ZMM31 => Size::Zword,
        }
    }
    #[deprecated]
    #[rustfmt::skip]
    pub const fn needs_evex(&self) -> bool {
        matches!(self,
            Self::XMM16 | Self::XMM18 | Self::XMM19 | Self::XMM20 |
            Self::XMM21 | Self::XMM22 | Self::XMM23 | Self::XMM24 |
            Self::XMM25 | Self::XMM26 | Self::XMM27 | Self::XMM28 |
            Self::XMM29 | Self::XMM30 | Self::XMM31 |
            Self::YMM16 | Self::YMM18 | Self::YMM19 | Self::YMM20 |
            Self::YMM21 | Self::YMM22 | Self::YMM23 | Self::YMM24 |
            Self::YMM25 | Self::YMM26 | Self::YMM27 | Self::YMM28 |
            Self::YMM29 | Self::YMM30 | Self::YMM31 |
            Self::ZMM0  | Self::ZMM1  | Self::ZMM2  | Self::ZMM3  |
            Self::ZMM4  | Self::ZMM5  | Self::ZMM6  | Self::ZMM7  |
            Self::ZMM8  | Self::ZMM9  | Self::ZMM10 | Self::ZMM11 |
            Self::ZMM12 | Self::ZMM13 | Self::ZMM14 | Self::ZMM15 |
            Self::ZMM16 | Self::ZMM18 | Self::ZMM19 | Self::ZMM20 |
            Self::ZMM21 | Self::ZMM22 | Self::ZMM23 | Self::ZMM24 |
            Self::ZMM25 | Self::ZMM26 | Self::ZMM27 | Self::ZMM28 |
            Self::ZMM29 | Self::ZMM30 | Self::ZMM31
        )
    }
    #[deprecated]
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
            Self::YMM12| Self::YMM13| Self::YMM14| Self::YMM15|
            Self::SIL  | Self::DIL  | Self::BPL  | Self::SPL  |
            Self::CR8  | Self::CR9  | Self::CR10 | Self::CR11 |
            Self::CR12 | Self::CR13 | Self::CR14 | Self::CR15 |
            Self::DR8  | Self::DR9  | Self::DR10 | Self::DR11 |
            Self::DR12 | Self::DR13 | Self::DR14 | Self::DR15
        )
    }
    #[rustfmt::skip]
    pub const fn to_byte(&self) -> u8 {
        match &self {
            Self::__ANY => 0b000,
            Self::ZMM0 | Self::ZMM8  | Self::K0   |
            Self::ES   | Self::MM0   | Self::XMM16| Self::XMM24 |
            Self::YMM16| Self::YMM24 | Self::ZMM16| Self::ZMM24 |
            Self::R8   | Self::R8B   | Self::R8W  | Self::R8D   |
            Self::XMM8 | Self::YMM8  | Self::AL   | Self::AX    |
            Self::EAX  | Self::CR0   | Self::CR8  | Self::DR0   |
            Self::DR8  | Self::RAX   | Self::XMM0 | Self::YMM0   => 0b000,

            Self::ZMM1 | Self::ZMM9  | Self::K1   |
            Self::CS   | Self::MM1   | Self::XMM17| Self::XMM25 |
            Self::YMM17| Self::YMM25 | Self::ZMM17| Self::ZMM25 |
            Self::R9   | Self::R9B   | Self::R9W  | Self::R9D   |
            Self::CL   | Self::CX    | Self::ECX  | Self::RCX   |
            Self::XMM1 | Self::YMM1  | Self::XMM9 | Self::CR1   |
            Self::YMM9 | Self::CR9   | Self::DR1  | Self::DR9    => 0b001,

            Self::ZMM2 | Self::ZMM10| Self::K2   |
            Self::SS   | Self::MM2  | Self::XMM18| Self::XMM26 |
            Self::YMM18| Self::YMM26| Self::ZMM18| Self::ZMM26 |
            Self::R10  | Self::R10B | Self::R10W | Self::R10D  |
            Self::DL   | Self::DX   | Self::EDX  | Self::XMM2  |
            Self::RDX  | Self::CR2  | Self::CR10 | Self::DR2   |
            Self::DR10 | Self::YMM2 | Self::XMM10| Self::YMM10  => 0b010,

            Self::ZMM3 | Self::ZMM11| Self::K3   |
            Self::DS   | Self::MM3  | Self::XMM19| Self::XMM27|
            Self::YMM19| Self::YMM27| Self::ZMM19| Self::ZMM27|
            Self::R11  | Self::R11B | Self::R11W | Self::R11D |
            Self::BL   | Self::BX   | Self::EBX  | Self::XMM3 |
            Self::RBX  | Self::CR3  | Self::CR11 | Self::DR3  |
            Self::DR11 | Self::YMM3 | Self::XMM11| Self::YMM11 => 0b011,

            Self::ZMM4 | Self::ZMM12| Self::K4   |
            Self::FS   | Self::MM4  | Self::XMM20| Self::XMM28|
            Self::YMM20| Self::YMM28| Self::ZMM20| Self::ZMM28|
            Self::R12  | Self::R12B | Self::R12W | Self::R12D |
            Self::AH   | Self::SP   | Self::ESP  | Self::XMM4 |
            Self::SPL  | Self::RSP  | Self::CR4  | Self::CR12 |
            Self::DR4  | Self::DR12 | Self::YMM4 | Self::XMM12|
            Self::YMM12                                       => 0b100,

            Self::ZMM5 | Self::ZMM13| Self::K5   |
            Self::GS   | Self::MM5  | Self::XMM21| Self::XMM29|
            Self::YMM21| Self::YMM29| Self::ZMM21| Self::ZMM29|
            Self::R13  | Self::R13B | Self::R13W | Self::R13D |
            Self::CH   | Self::BP   | Self::EBP  | Self::XMM5 |
            Self::BPL  | Self::RBP  | Self::CR5  | Self::CR13 |
            Self::DR5  | Self::DR13 | Self::YMM5 | Self::XMM13|
            Self::YMM13                                       => 0b101,

            Self::ZMM6  | Self::ZMM14| Self::K6   |
            Self::R14   | Self::R14B | Self::R14W | Self::R14D |
            Self::YMM22 | Self::YMM30| Self::ZMM22| Self::ZMM30|
            Self::DH    | Self::SI   | Self::ESI  | Self::XMM6 |
            Self::SIL   | Self::RSI  | Self::CR6  | Self::CR14 |
            Self::DR6   | Self::DR14 | Self::YMM6 | Self::XMM14|
            Self::YMM14 | Self::MM6  | Self::XMM22| Self::XMM30 => 0b110,

            Self::ZMM7  | Self::ZMM15| Self::K7   |
            Self::R15   | Self::R15B | Self::R15W | Self::R15D |
            Self::YMM23 | Self::YMM31| Self::ZMM23| Self::ZMM31|
            Self::BH    | Self::DI   | Self::EDI  | Self::XMM7 |
            Self::DIL   | Self::RDI  | Self::CR7  | Self::CR15 |
            Self::DR7   | Self::DR15 | Self::YMM7 | Self::XMM15|
            Self::YMM15 | Self::MM7  | Self::XMM23| Self::XMM31 => 0b111,

            Self::IP | Self::EIP | Self::RIP => 0b000
        }
    }
    pub fn is_mask(&self) -> bool {
        self.purpose() == Purpose::Mask
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
            Self::XMM0 | Self::XMM1 | Self::XMM2 | Self::XMM3 |
            Self::XMM4 | Self::XMM5 | Self::XMM6 | Self::XMM7 |
            Self::XMM8 | Self::XMM9 | Self::XMM10| Self::XMM11|
            Self::XMM12| Self::XMM13| Self::XMM14| Self::XMM15|
            Self::XMM16| Self::XMM17| Self::XMM18| Self::XMM19|
            Self::XMM20| Self::XMM21| Self::XMM22| Self::XMM23|
            Self::XMM24| Self::XMM25| Self::XMM26| Self::XMM27|
            Self::XMM28| Self::XMM29| Self::XMM30| Self::XMM31 => Purpose::F128,
            Self::YMM0 | Self::YMM1 | Self::YMM2 | Self::YMM3 |
            Self::YMM4 | Self::YMM5 | Self::YMM6 | Self::YMM7 |
            Self::YMM8 | Self::YMM9 | Self::YMM10| Self::YMM11|
            Self::YMM12| Self::YMM13| Self::YMM14| Self::YMM15|
            Self::YMM16| Self::YMM17| Self::YMM18| Self::YMM19|
            Self::YMM20| Self::YMM21| Self::YMM22| Self::YMM23|
            Self::YMM24| Self::YMM25| Self::YMM26| Self::YMM27|
            Self::YMM28| Self::YMM29| Self::YMM30| Self::YMM31 => Purpose::F256,
            Self::ZMM0 | Self::ZMM1 | Self::ZMM2 | Self::ZMM3 |
            Self::ZMM4 | Self::ZMM5 | Self::ZMM6 | Self::ZMM7 |
            Self::ZMM8 | Self::ZMM9 | Self::ZMM10| Self::ZMM11|
            Self::ZMM12| Self::ZMM13| Self::ZMM14| Self::ZMM15|
            Self::ZMM16| Self::ZMM17| Self::ZMM18| Self::ZMM19|
            Self::ZMM20| Self::ZMM21| Self::ZMM22| Self::ZMM23|
            Self::ZMM24| Self::ZMM25| Self::ZMM26| Self::ZMM27|
            Self::ZMM28| Self::ZMM29| Self::ZMM30| Self::ZMM31 => Purpose::F512,
            Self::MM0 | Self::MM1 | Self::MM2 | Self::MM3  |
            Self::MM4 | Self::MM5 | Self::MM6 | Self::MM7  => Purpose::Mmx,
            Self::K0 | Self::K1 | Self::K2 | Self::K3  |
            Self::K4 | Self::K5 | Self::K6 | Self::K7  => Purpose::Mask,
        }
    }
    // For mksek(), se() and de():
    // 0b000_XX_YYYY_ZZZZ_AAA
    // X0 - evex extension
    // X1 - extended register
    //
    // YYYY - size
    // ZZZZ - purpose
    // AAA  - register code
    pub const fn mksek(eext: bool, ext: bool, sz: u16, prp: u16, cd: u16) -> u16 {
        (eext as u16) << 12 | (ext as u16) << 11 | sz << 7 | prp << 3 | cd
    }
    pub const fn se(&self) -> u16 {
        let mut tret = 0;

        tret |= self.to_byte() as u16;

        tret |= (self.purpose() as u16) << 3;
        tret |= (self.size() as u16) << 7;

        let tmp = self.get_ext_bits();

        tret |= (tmp[0] as u16) << 12 | (tmp[1] as u16) << 11;

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
        let extr = (data & 0b11 << 11) >> 11;

        match purp {
            Purpose::__ANY => Self::__ANY,
            Purpose::IPtr => match size {
                Size::Word => Register::IP,
                Size::Dword => Register::EIP,
                Size::Qword => Register::RIP,
                _ => Register::__ANY,
            },
            Purpose::Sgmnt => match code {
                0b000 => Self::ES,
                0b001 => Self::CS,
                0b010 => Self::SS,
                0b011 => Self::DS,
                0b100 => Self::FS,
                0b101 => Self::GS,
                _ => Self::__ANY,
            },
            Purpose::Mask => match code {
                0b000 => Self::K0,
                0b001 => Self::K1,
                0b010 => Self::K2,
                0b011 => Self::K3,
                0b100 => Self::K4,
                0b101 => Self::K5,
                0b110 => Self::K6,
                0b111 => Self::K7,
                _ => Self::__ANY,
            },
            Purpose::Mmx => match code {
                0b000 => Self::MM0,
                0b001 => Self::MM1,
                0b010 => Self::MM2,
                0b011 => Self::MM3,
                0b100 => Self::MM4,
                0b101 => Self::MM5,
                0b110 => Self::MM6,
                0b111 => Self::MM7,
                _ => Self::__ANY,
            },
            Purpose::F256 => match (extr & 0b11, code) {
                (0b00, 0b000) => Self::YMM0,
                (0b00, 0b001) => Self::YMM1,
                (0b00, 0b010) => Self::YMM2,
                (0b00, 0b011) => Self::YMM3,
                (0b00, 0b100) => Self::YMM4,
                (0b00, 0b101) => Self::YMM5,
                (0b00, 0b110) => Self::YMM6,
                (0b00, 0b111) => Self::YMM7,
                (0b01, 0b000) => Self::YMM8,
                (0b01, 0b001) => Self::YMM9,
                (0b01, 0b010) => Self::YMM10,
                (0b01, 0b011) => Self::YMM11,
                (0b01, 0b100) => Self::YMM12,
                (0b01, 0b101) => Self::YMM13,
                (0b01, 0b110) => Self::YMM14,
                (0b01, 0b111) => Self::YMM15,
                (0b10, 0b000) => Self::YMM16,
                (0b10, 0b001) => Self::YMM17,
                (0b10, 0b010) => Self::YMM18,
                (0b10, 0b011) => Self::YMM19,
                (0b10, 0b100) => Self::YMM20,
                (0b10, 0b101) => Self::YMM21,
                (0b10, 0b110) => Self::YMM22,
                (0b10, 0b111) => Self::YMM23,
                (0b11, 0b000) => Self::YMM24,
                (0b11, 0b001) => Self::YMM25,
                (0b11, 0b010) => Self::YMM26,
                (0b11, 0b011) => Self::YMM27,
                (0b11, 0b100) => Self::YMM28,
                (0b11, 0b101) => Self::YMM29,
                (0b11, 0b110) => Self::YMM30,
                (0b11, 0b111) => Self::YMM31,
                _ => Self::__ANY,
            },
            Purpose::F512 => match (extr & 0b11, code) {
                (0b00, 0b000) => Self::ZMM0,
                (0b00, 0b001) => Self::ZMM1,
                (0b00, 0b010) => Self::ZMM2,
                (0b00, 0b011) => Self::ZMM3,
                (0b00, 0b100) => Self::ZMM4,
                (0b00, 0b101) => Self::ZMM5,
                (0b00, 0b110) => Self::ZMM6,
                (0b00, 0b111) => Self::ZMM7,
                (0b01, 0b000) => Self::ZMM8,
                (0b01, 0b001) => Self::ZMM9,
                (0b01, 0b010) => Self::ZMM10,
                (0b01, 0b011) => Self::ZMM11,
                (0b01, 0b100) => Self::ZMM12,
                (0b01, 0b101) => Self::ZMM13,
                (0b01, 0b110) => Self::ZMM14,
                (0b01, 0b111) => Self::ZMM15,
                (0b10, 0b000) => Self::ZMM16,
                (0b10, 0b001) => Self::ZMM17,
                (0b10, 0b010) => Self::ZMM18,
                (0b10, 0b011) => Self::ZMM19,
                (0b10, 0b100) => Self::ZMM20,
                (0b10, 0b101) => Self::ZMM21,
                (0b10, 0b110) => Self::ZMM22,
                (0b10, 0b111) => Self::ZMM23,
                (0b11, 0b000) => Self::ZMM24,
                (0b11, 0b001) => Self::ZMM25,
                (0b11, 0b010) => Self::ZMM26,
                (0b11, 0b011) => Self::ZMM27,
                (0b11, 0b100) => Self::ZMM28,
                (0b11, 0b101) => Self::ZMM29,
                (0b11, 0b110) => Self::ZMM30,
                (0b11, 0b111) => Self::ZMM31,
                _ => Self::__ANY,
            },
            Purpose::F128 => match (extr & 0b11, code) {
                (0b00, 0b000) => Self::XMM0,
                (0b00, 0b001) => Self::XMM1,
                (0b00, 0b010) => Self::XMM2,
                (0b00, 0b011) => Self::XMM3,
                (0b00, 0b100) => Self::XMM4,
                (0b00, 0b101) => Self::XMM5,
                (0b00, 0b110) => Self::XMM6,
                (0b00, 0b111) => Self::XMM7,
                (0b01, 0b000) => Self::XMM8,
                (0b01, 0b001) => Self::XMM9,
                (0b01, 0b010) => Self::XMM10,
                (0b01, 0b011) => Self::XMM11,
                (0b01, 0b100) => Self::XMM12,
                (0b01, 0b101) => Self::XMM13,
                (0b01, 0b110) => Self::XMM14,
                (0b01, 0b111) => Self::XMM15,
                (0b10, 0b000) => Self::XMM16,
                (0b10, 0b001) => Self::XMM17,
                (0b10, 0b010) => Self::XMM18,
                (0b10, 0b011) => Self::XMM19,
                (0b10, 0b100) => Self::XMM20,
                (0b10, 0b101) => Self::XMM21,
                (0b10, 0b110) => Self::XMM22,
                (0b10, 0b111) => Self::XMM23,
                (0b11, 0b000) => Self::XMM24,
                (0b11, 0b001) => Self::XMM25,
                (0b11, 0b010) => Self::XMM26,
                (0b11, 0b011) => Self::XMM27,
                (0b11, 0b100) => Self::XMM28,
                (0b11, 0b101) => Self::XMM29,
                (0b11, 0b110) => Self::XMM30,
                (0b11, 0b111) => Self::XMM31,
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
            Self::F512 => "zmm".to_string(),
            Self::Mask => "k".to_string(),
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Register {
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
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
    use Register::*;
    let r = str.as_bytes();
    match r.len() {
        2 => match r[0] {
            b'e' => match r[1] {
                b's' => s(ES),
                _ => N,
            },
            b'f' => match r[1] {
                b's' => s(FS),
                _ => N,
            },
            b'g' => match r[1] {
                b's' => s(GS),
                _ => N,
            },
            b'i' => match r[1] {
                b'p' => s(IP),
                _ => N,
            },
            b'a' => match r[1] {
                b'h' => s(AH),
                b'l' => s(AL),
                b'x' => s(AX),
                _ => N,
            },
            b'b' => match r[1] {
                b'h' => s(BH),
                b'l' => s(BL),
                b'p' => s(BP),
                b'x' => s(BX),
                _ => N,
            },
            b'c' => match r[1] {
                b'h' => s(CH),
                b'l' => s(CL),
                b's' => s(CS),
                b'x' => s(CX),
                _ => N,
            },
            b'd' => match r[1] {
                b'h' => s(DH),
                b'i' => s(DI),
                b'l' => s(DL),
                b's' => s(DS),
                b'x' => s(DX),
                _ => N,
            },
            b'k' => match r[1] {
                b'0' => s(K0),
                b'1' => s(K1),
                b'2' => s(K2),
                b'3' => s(K3),
                b'4' => s(K4),
                b'5' => s(K5),
                b'6' => s(K6),
                b'7' => s(K7),
                _ => N,
            },
            b'r' => match r[1] {
                b'8' => s(R8),
                b'9' => s(R9),
                _ => N,
            },
            b's' => match r[1] {
                b'i' => s(SI),
                b'p' => s(SP),
                b's' => s(SS),
                _ => N,
            },
            _ => N,
        },
        3 => match r[0] {
            b'b' => match r[1] {
                b'p' => match r[2] {
                    b'l' => s(BPL),
                    _ => N,
                },
                _ => N,
            },
            b'c' => match r[1] {
                b'r' => match r[2] {
                    b'0' => s(CR0),
                    b'1' => s(CR1),
                    b'2' => s(CR2),
                    b'3' => s(CR3),
                    b'4' => s(CR4),
                    b'5' => s(CR5),
                    b'6' => s(CR6),
                    b'7' => s(CR7),
                    b'8' => s(CR8),
                    b'9' => s(CR9),
                    _ => N,
                },
                _ => N,
            },
            b'm' => match r[1] {
                b'm' => match r[2] {
                    b'0' => s(MM0),
                    b'1' => s(MM1),
                    b'2' => s(MM2),
                    b'3' => s(MM3),
                    b'4' => s(MM4),
                    b'5' => s(MM5),
                    b'6' => s(MM6),
                    b'7' => s(MM7),
                    _ => N,
                },
                _ => N,
            },
            b'd' => match r[1] {
                b'i' => match r[2] {
                    b'l' => s(DIL),
                    _ => N,
                },
                b'r' => match r[2] {
                    b'0' => s(DR0),
                    b'1' => s(DR1),
                    b'2' => s(DR2),
                    b'3' => s(DR3),
                    b'4' => s(DR4),
                    b'5' => s(DR5),
                    b'6' => s(DR6),
                    b'7' => s(DR7),
                    b'8' => s(DR8),
                    b'9' => s(DR9),
                    _ => N,
                },
                _ => N,
            },
            b'e' => match r[1] {
                b'a' => match r[2] {
                    b'x' => s(EAX),
                    _ => N,
                },
                b'c' => match r[2] {
                    b'x' => s(ECX),
                    _ => N,
                },
                b'i' => match r[2] {
                    b'p' => s(EIP),
                    _ => N,
                },
                b'b' => match r[2] {
                    b'p' => s(EBP),
                    b'x' => s(EBX),
                    _ => N,
                },
                b'd' => match r[2] {
                    b'i' => s(EDI),
                    b'x' => s(EDX),
                    _ => N,
                },
                b's' => match r[2] {
                    b'i' => s(ESI),
                    b'p' => s(ESP),
                    _ => N,
                },
                _ => N,
            },
            b'r' => match r[1] {
                b'a' => match r[2] {
                    b'x' => s(RAX),
                    _ => N,
                },
                b'c' => match r[2] {
                    b'x' => s(RCX),
                    _ => N,
                },
                b'i' => match r[2] {
                    b'p' => s(RIP),
                    _ => N,
                },
                b'1' => match r[2] {
                    b'0' => s(R10),
                    b'1' => s(R11),
                    b'2' => s(R12),
                    b'3' => s(R13),
                    b'4' => s(R14),
                    b'5' => s(R15),
                    _ => N,
                },
                b'8' => match r[2] {
                    b'b' => s(R8B),
                    b'd' => s(R8D),
                    b'w' => s(R8W),
                    _ => N,
                },
                b'9' => match r[2] {
                    b'b' => s(R9B),
                    b'd' => s(R9D),
                    b'w' => s(R9W),
                    _ => N,
                },
                b'b' => match r[2] {
                    b'p' => s(RBP),
                    b'x' => s(RBX),
                    _ => N,
                },
                b'd' => match r[2] {
                    b'i' => s(RDI),
                    b'x' => s(RDX),
                    _ => N,
                },
                b's' => match r[2] {
                    b'i' => s(RSI),
                    b'p' => s(RSP),
                    _ => N,
                },
                _ => N,
            },
            b's' => match r[1] {
                b'i' => match r[2] {
                    b'l' => s(SIL),
                    _ => N,
                },
                b'p' => match r[2] {
                    b'l' => s(SPL),
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
                        b'0' => s(CR10),
                        b'1' => s(CR11),
                        b'2' => s(CR12),
                        b'3' => s(CR13),
                        b'4' => s(CR14),
                        b'5' => s(CR15),
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'd' => match r[1] {
                b'r' => match r[2] {
                    b'1' => match r[3] {
                        b'0' => s(DR10),
                        b'1' => s(DR11),
                        b'2' => s(DR12),
                        b'3' => s(DR13),
                        b'4' => s(DR14),
                        b'5' => s(DR15),
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'r' => match r[1] {
                b'1' => match r[2] {
                    b'0' => match r[3] {
                        b'b' => s(R10B),
                        b'd' => s(R10D),
                        b'w' => s(R10W),
                        _ => N,
                    },
                    b'1' => match r[3] {
                        b'b' => s(R11B),
                        b'd' => s(R11D),
                        b'w' => s(R11W),
                        _ => N,
                    },
                    b'2' => match r[3] {
                        b'b' => s(R12B),
                        b'd' => s(R12D),
                        b'w' => s(R12W),
                        _ => N,
                    },
                    b'3' => match r[3] {
                        b'b' => s(R13B),
                        b'd' => s(R13D),
                        b'w' => s(R13W),
                        _ => N,
                    },
                    b'4' => match r[3] {
                        b'b' => s(R14B),
                        b'd' => s(R14D),
                        b'w' => s(R14W),
                        _ => N,
                    },
                    b'5' => match r[3] {
                        b'b' => s(R15B),
                        b'd' => s(R15D),
                        b'w' => s(R15W),
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'x' => match r[1] {
                b'm' => match r[2] {
                    b'm' => match r[3] {
                        b'0' => s(XMM0),
                        b'1' => s(XMM1),
                        b'2' => s(XMM2),
                        b'3' => s(XMM3),
                        b'4' => s(XMM4),
                        b'5' => s(XMM5),
                        b'6' => s(XMM6),
                        b'7' => s(XMM7),
                        b'8' => s(XMM8),
                        b'9' => s(XMM9),
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'y' => match r[1] {
                b'm' => match r[2] {
                    b'm' => match r[3] {
                        b'0' => s(YMM0),
                        b'1' => s(YMM1),
                        b'2' => s(YMM2),
                        b'3' => s(YMM3),
                        b'4' => s(YMM4),
                        b'5' => s(YMM5),
                        b'6' => s(YMM6),
                        b'7' => s(YMM7),
                        b'8' => s(YMM8),
                        b'9' => s(YMM9),
                        _ => N,
                    },
                    _ => N,
                },
                _ => N,
            },
            b'z' => match r[1] {
                b'm' => match r[2] {
                    b'm' => match r[3] {
                        b'0' => s(ZMM0),
                        b'1' => s(ZMM1),
                        b'2' => s(ZMM2),
                        b'3' => s(ZMM3),
                        b'4' => s(ZMM4),
                        b'5' => s(ZMM5),
                        b'6' => s(ZMM6),
                        b'7' => s(ZMM7),
                        b'8' => s(ZMM8),
                        b'9' => s(ZMM9),
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
                            b'0' => s(XMM10),
                            b'1' => s(XMM11),
                            b'2' => s(XMM12),
                            b'3' => s(XMM13),
                            b'4' => s(XMM14),
                            b'5' => s(XMM15),
                            b'6' => s(XMM16),
                            b'7' => s(XMM17),
                            b'8' => s(XMM18),
                            b'9' => s(XMM19),
                            _ => N,
                        },
                        b'2' => match r[4] {
                            b'0' => s(XMM20),
                            b'1' => s(XMM21),
                            b'2' => s(XMM22),
                            b'3' => s(XMM23),
                            b'4' => s(XMM24),
                            b'5' => s(XMM25),
                            b'6' => s(XMM26),
                            b'7' => s(XMM27),
                            b'8' => s(XMM28),
                            b'9' => s(XMM29),
                            _ => N,
                        },
                        b'3' => match r[4] {
                            b'0' => s(XMM30),
                            b'1' => s(XMM31),
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
                            b'0' => s(YMM10),
                            b'1' => s(YMM11),
                            b'2' => s(YMM12),
                            b'3' => s(YMM13),
                            b'4' => s(YMM14),
                            b'5' => s(YMM15),
                            b'6' => s(YMM16),
                            b'7' => s(YMM17),
                            b'8' => s(YMM18),
                            b'9' => s(YMM19),
                            _ => N,
                        },
                        b'2' => match r[4] {
                            b'0' => s(YMM20),
                            b'1' => s(YMM21),
                            b'2' => s(YMM22),
                            b'3' => s(YMM23),
                            b'4' => s(YMM24),
                            b'5' => s(YMM25),
                            b'6' => s(YMM26),
                            b'7' => s(YMM27),
                            b'8' => s(YMM28),
                            b'9' => s(YMM29),
                            _ => N,
                        },
                        b'3' => match r[4] {
                            b'0' => s(YMM30),
                            b'1' => s(YMM31),
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
                            b'0' => s(ZMM10),
                            b'1' => s(ZMM11),
                            b'2' => s(ZMM12),
                            b'3' => s(ZMM13),
                            b'4' => s(ZMM14),
                            b'5' => s(ZMM15),
                            b'6' => s(ZMM16),
                            b'7' => s(ZMM17),
                            b'8' => s(ZMM18),
                            b'9' => s(ZMM19),
                            _ => N,
                        },
                        b'2' => match r[4] {
                            b'0' => s(ZMM20),
                            b'1' => s(ZMM21),
                            b'2' => s(ZMM22),
                            b'3' => s(ZMM23),
                            b'4' => s(ZMM24),
                            b'5' => s(ZMM25),
                            b'6' => s(ZMM26),
                            b'7' => s(ZMM27),
                            b'8' => s(ZMM28),
                            b'9' => s(ZMM29),
                            _ => N,
                        },
                        b'3' => match r[4] {
                            b'0' => s(ZMM30),
                            b'1' => s(ZMM31),
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
