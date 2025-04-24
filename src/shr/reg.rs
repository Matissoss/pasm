// rasmx86_64 - src/shr/reg.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    conf::FAST_MODE,
    shr::{
        atype::{
            ToAType,
            AType
        },
        size::Size,
    }
};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Register{
    AL , BL , CL , DL ,
    SPL, BPL, SIL, DIL,
    AH , BH , CH , DH,

    AX , BX , CX , DX ,
    SP , BP , SI , DI ,

    EAX, EBX, ECX, EDX,
    ESP, EBP, ESI, EDI,

    RAX, RBX, RCX, RDX,
    RSP, RBP, RSI, RDI,

    R8 , R9 , R10, R11,
    R12, R13, R14, R15,

    R8D , R9D , R10D, R11D,
    R12D, R13D, R14D, R15D,

    R8W , R9W , R10W, R11W,
    R12W, R13W, R14W, R15W,
    
    R8B , R9B , R10B, R11B,
    R12B, R13B, R14B, R15B,

    CR0 , CR2 , CR3 , CR4 , 
    CR8 , DR0 , DR1 , DR2 , 
    DR3 , DR6 , DR7 , 

    RIP , EIP , IP  ,
    // CURRENTLY UNSUPPORTED:
    //  SIMD
    XMM0 , XMM1 , XMM2 , XMM3 , 
    XMM4 , XMM5 , XMM6 , XMM7 , 
    XMM8 , XMM9 , XMM10, XMM11,
    XMM12, XMM13, XMM14, XMM15,

    //  AVX
    YMM0, YMM1 , YMM2 , YMM3 , 
    YMM4, YMM5 , YMM6 , YMM7 , 
    YMM8, YMM9 , YMM10, YMM11,
    YMM12,YMM13, YMM14, YMM15,
}

#[inline(always)]
fn reg_ie(str: &str, tgt: &str, reg: Register) -> Result<Register, ()> {
    if FAST_MODE {
        return Ok(reg);
    }
    else {
        if str == tgt {
            return Ok(reg);
        }
        else {
            return Err(());
        }
    }
}

impl FromStr for Register{
    type Err = ();
    fn from_str(str: &str) -> Result<Self, <Self as FromStr>::Err>{
        let byte_str = str.as_bytes();
        return match byte_str.len(){
            1 => Err(()),
            2 => {
                match byte_str[1] as char{
                    'i' => {
                        match byte_str[0] as char {
                            's' => reg_ie(str, "si", Register::SI),
                            'd' => reg_ie(str, "di", Register::DI),
                            _   => Err(())
                        }
                    },
                    'l' => {
                        match byte_str[0] as char {
                            'a' => reg_ie(str, "al", Register::AL),
                            'b' => reg_ie(str, "bl", Register::BL),
                            'c' => reg_ie(str, "cl", Register::CL),
                            'd' => reg_ie(str, "dl", Register::DL),
                            _   => Err(())
                        }
                    }
                    'x' => {
                        match byte_str[0] as char {
                            'a' => reg_ie(str, "ax", Register::AX),
                            'b' => reg_ie(str, "bx", Register::BX),
                            'c' => reg_ie(str, "cx", Register::CX),
                            'd' => reg_ie(str, "dx", Register::DX),
                            _   => Err(())
                        }
                    }
                    'h' => {
                        match byte_str[0] as char {
                            'a' => reg_ie(str, "ah", Register::AH),
                            'b' => reg_ie(str, "bh", Register::BH),
                            'c' => reg_ie(str, "ch", Register::CH),
                            'd' => reg_ie(str, "dh", Register::DH),
                            _   => Err(())
                        }
                    },
                    'p' => {
                        match byte_str[0] as char{
                            'i' => reg_ie(str, "ip", Register::IP),
                            'b' => reg_ie(str, "bp", Register::BP),
                            's' => reg_ie(str, "sp", Register::SP),
                            _   => Err(())
                        }
                    },
                    '8' => reg_ie(str, "r8", Register::R8),
                    '9' => reg_ie(str, "r9", Register::R9),
                    _ => Err(())
                }
            },
            // prev = 2; byte_str.len()
            3 => {
                match byte_str[0] as char {
                    'r' => {
                        match byte_str[1] as char{
                            'a' => reg_ie(str, "rax", Register::RAX),
                            'b' => {
                                match byte_str[2] as char {
                                    'x' => reg_ie(str,"rbx", Register::RBX),
                                    'p' => reg_ie(str,"rbp", Register::RBP),
                                    _ => Err(())
                                }
                            },
                            'c' => return Ok(Register::RCX),
                            'd' => {
                                match byte_str[2] as char {
                                    'x' => reg_ie(str,"rdx", Register::RDX),
                                    'i' => reg_ie(str,"rdi", Register::RDI),
                                    _ => Err(())
                                }
                            },
                            's' => {
                                match byte_str[2] as char {
                                    'p' => reg_ie(str,"rsp", Register::RSP),
                                    'i' => reg_ie(str,"rsi", Register::RSI),
                                    _ => Err(())
                                }
                            }
                            '1' => {
                                match byte_str[2] as char {
                                    '0' => reg_ie(str,"r10", Register::R10),
                                    '1' => reg_ie(str,"r11", Register::R11),
                                    '2' => reg_ie(str,"r12", Register::R12),
                                    '3' => reg_ie(str,"r13", Register::R13),
                                    '4' => reg_ie(str,"r14", Register::R14),
                                    '5' => reg_ie(str,"r15", Register::R15),
                                    _ => Err(())
                                }
                            }
                            '8' => {
                                match byte_str[2] as char {
                                    'b' => reg_ie(str,"r8b", Register::R8B),
                                    'w' => reg_ie(str,"r8w", Register::R8W),
                                    'd' => reg_ie(str,"r8d", Register::R8D),
                                    _   => Err(())
                                }
                            }
                            '9' => {
                                match byte_str[2] as char {
                                    'b' => reg_ie(str,"r9b", Register::R9B),
                                    'w' => reg_ie(str,"r9w", Register::R9W),
                                    'd' => reg_ie(str,"r9d", Register::R9D),
                                    _ => Err(())
                                }
                            }
                            'i' => reg_ie(str, "rip", Register::RIP),
                            _ => Err(())
                        }
                    },
                    // prev = 'r'; byte_str[0]
                    'e' => {
                        match byte_str[1] as char {
                            'a' => reg_ie(str,"eax",Register::EAX),
                            'b' => {
                                match byte_str[2] as char {
                                    'p' => reg_ie(str,"ebp",Register::EBP),
                                    'x' => reg_ie(str,"ebx",Register::EBX),
                                    _ => Err(())
                                }
                            },
                            'c' => reg_ie(str,"ecx",Register::ECX),
                            'd' => {
                                match byte_str[2] as char {
                                    'i' => reg_ie(str,"edi",Register::EDI),
                                    'x' => reg_ie(str,"edx",Register::EDX),
                                    _ => Err(())
                                }
                            }
                            's' => {
                                match byte_str[2] as char {
                                    'i' => reg_ie(str,"edi",Register::ESI),
                                    'p' => reg_ie(str,"edx",Register::ESP),
                                    _ => Err(())
                                }
                            }
                            'i' => reg_ie(str, "eip", Register::EIP),
                            _ => Err(())
                        }
                    }
                    // prev = 'e'; byte_str[0]
                    's' => {
                        match byte_str[1] as char {
                            'p' => reg_ie(str,"spl",Register::SPL),
                            'i' => reg_ie(str,"sil",Register::SIL),
                            _ => Err(())
                        }
                    }
                    'b' => reg_ie(str, "bpl", Register::BPL),
                    'c' => {
                        match byte_str[2] as char {
                            '0' => reg_ie(str,"cr0",Register::CR0),
                            '2' => reg_ie(str,"cr2",Register::CR2),
                            '3' => reg_ie(str,"cr3",Register::CR3),
                            '4' => reg_ie(str,"cr4",Register::CR4),
                            '8' => reg_ie(str,"cr8",Register::CR8),
                            _ => Err(())
                        }
                    }
                    'd' => {
                        match byte_str[2] as char{
                            'i' => reg_ie(str,"dil",Register::DIL),
                            '0' => reg_ie(str,"dr0",Register::DR0),
                            '1' => reg_ie(str,"dr1",Register::DR1),
                            '2' => reg_ie(str,"dr2",Register::DR2),
                            '3' => reg_ie(str,"dr3",Register::DR3),
                            '6' => reg_ie(str,"dr6",Register::DR6),
                            '7' => reg_ie(str,"dr7",Register::DR7),
                            _ => Err(())
                        }
                    },
                    _ => Err(())
                }
            }
            // prev = 3; byte_str.len()
            4 => {
                match byte_str[0] as char {
                    'r' => {
                        match byte_str[2] as char{
                            '0' => {
                                match byte_str[3] as char {
                                    'd' => reg_ie(str,"r10d",Register::R10D),
                                    'b' => reg_ie(str,"r10b",Register::R10B),
                                    'w' => reg_ie(str,"r10w",Register::R10W),
                                    _ => Err(())
                                }
                            },
                            '1' => {
                                match byte_str[3] as char {
                                    'd' => reg_ie(str,"r11d",Register::R11D),
                                    'b' => reg_ie(str,"r11b",Register::R11B),
                                    'w' => reg_ie(str,"r11w",Register::R11W),
                                    _ => Err(())
                                }
                            },
                            '2' => {
                                match byte_str[3] as char {
                                    'd' => reg_ie(str,"r12d",Register::R12D),
                                    'b' => reg_ie(str,"r12b",Register::R12B),
                                    'w' => reg_ie(str,"r12w",Register::R12W),
                                    _ => Err(())
                                }
                            },
                            '3' => {
                                match byte_str[3] as char {
                                    'd' => reg_ie(str,"r13d",Register::R13D),
                                    'b' => reg_ie(str,"r13b",Register::R13B),
                                    'w' => reg_ie(str,"r13w",Register::R13W),
                                    _ => Err(())
                                }
                            },
                            '4' => {
                                match byte_str[3] as char {
                                    'b' => reg_ie(str,"r14b",Register::R14B),
                                    'w' => reg_ie(str,"r14w",Register::R14W),
                                    'd' => reg_ie(str,"r14d",Register::R14D),
                                    _ => Err(())
                                }
                            },
                            '5' => {
                                match byte_str[3] as char {
                                    'b' => reg_ie(str,"r15b",Register::R15B),
                                    'w' => reg_ie(str,"r15w",Register::R15W),
                                    'd' => reg_ie(str,"r15d",Register::R15D),
                                    _ => Err(())
                                }
                            },
                            _ => Err(())
                        }
                    }
                    'x' => {
                        match byte_str[3] as char {
                            '0' => reg_ie(str,"xmm0", Register::XMM0),
                            '1' => reg_ie(str,"xmm1", Register::XMM1),
                            '2' => reg_ie(str,"xmm2", Register::XMM2),
                            '3' => reg_ie(str,"xmm3", Register::XMM3),
                            '4' => reg_ie(str,"xmm4", Register::XMM4),
                            '5' => reg_ie(str,"xmm5", Register::XMM5),
                            '6' => reg_ie(str,"xmm6", Register::XMM6),
                            '7' => reg_ie(str,"xmm7", Register::XMM7),
                            '8' => reg_ie(str,"xmm8", Register::XMM8),
                            '9' => reg_ie(str,"xmm9", Register::XMM9),
                            _   => return Err(())
                        }
                    }
                    'y' => {
                        match byte_str[3] as char {
                            '0' => reg_ie(str,"ymm0", Register::YMM0),
                            '1' => reg_ie(str,"ymm1", Register::YMM1),
                            '2' => reg_ie(str,"ymm2", Register::YMM2),
                            '3' => reg_ie(str,"ymm3", Register::YMM3),
                            '4' => reg_ie(str,"ymm4", Register::YMM4),
                            '5' => reg_ie(str,"ymm5", Register::YMM5),
                            '6' => reg_ie(str,"ymm6", Register::YMM6),
                            '7' => reg_ie(str,"ymm7", Register::YMM7),
                            '8' => reg_ie(str,"ymm8", Register::YMM8),
                            '9' => reg_ie(str,"ymm9", Register::YMM9),
                            _   => return Err(())
                        }
                    }
                    _ => Err(())
                }
            },
            5 => {
                match byte_str[0] as char {
                    'x' => {
                        match byte_str[4] as char {
                            '0' => reg_ie(str,"xmm10", Register::XMM10),
                            '1' => reg_ie(str,"xmm11", Register::XMM11),
                            '2' => reg_ie(str,"xmm12", Register::XMM12),
                            '3' => reg_ie(str,"xmm13", Register::XMM13),
                            '4' => reg_ie(str,"xmm14", Register::XMM14),
                            '5' => reg_ie(str,"xmm15", Register::XMM15),
                            _ => Err(())
                        }
                    }
                    'y' => {
                        match byte_str[4] as char {
                            '0' => reg_ie(str,"ymm10", Register::YMM10),
                            '1' => reg_ie(str,"ymm11", Register::YMM11),
                            '2' => reg_ie(str,"ymm12", Register::YMM12),
                            '3' => reg_ie(str,"ymm13", Register::YMM13),
                            '4' => reg_ie(str,"ymm14", Register::YMM14),
                            '5' => reg_ie(str,"ymm15", Register::YMM15),
                            _ => Err(())
                        }
                    },
                    _ => Err(())
                }
            }
            _ => Err(())
        };
    }
}

impl ToAType for Register{
    fn atype(&self) -> AType{
        return AType::Reg(self.size());
    }
}

impl Register{
    pub fn size(&self) -> Size {
        match self{
            Self::AL  |Self::BL  |Self::CL  |Self::DL   |
            Self::AH  |Self::BH  |Self::CH  |Self::DH   |
            Self::SPL |Self::BPL |Self::SIL |Self::DIL  |
            Self::R8B |Self::R9B |Self::R10B|Self::R11B |
            Self::R12B|Self::R13B|Self::R14B|Self::R15B => Size::Byte,

            Self::AX  |Self::BX  |Self::CX  |Self::DX   |
            Self::SP  |Self::BP  |Self::SI  |Self::DI   |
            Self::IP  |
            Self::R8W |Self::R9W |Self::R10W|Self::R11W |
            Self::R12W|Self::R13W|Self::R14W|Self::R15W => Size::Word,

            Self::EAX |Self::EBX |Self::ECX |Self::EDX  |
            Self::ESP |Self::EBP |Self::ESI |Self::EDI  |
            Self::EIP |
            Self::CR0 |Self::CR2 |Self::CR3 |Self::CR4  |
            Self::CR8 |Self::DR0 |Self::DR1 |Self::DR2  |
            Self::DR3 |Self::DR6 |Self::DR7 |
            Self::R8D |Self::R9D |Self::R10D|Self::R11D |
            Self::R12D|Self::R13D|Self::R14D|Self::R15D => Size::Dword,

            Self::RAX |Self::RBX |Self::RCX |Self::RDX  |
            Self::RSP |Self::RBP |Self::RSI |Self::RDI  |
            Self::R8  |Self::R9  |Self::R10 |Self::R11  |
            Self::RIP |
            Self::R12 |Self::R13 |Self::R14 |Self::R15  => Size::Qword,

            Self::XMM0 |Self::XMM1 |Self::XMM2 |Self::XMM3  |
            Self::XMM4 |Self::XMM5 |Self::XMM6 |Self::XMM7  |
            Self::XMM8 |Self::XMM9 |Self::XMM10|Self::XMM11 |
            Self::XMM12|Self::XMM13|Self::XMM14|Self::XMM15 => Size::Xword,

            Self::YMM0 |Self::YMM1 |Self::YMM2 |Self::YMM3  |
            Self::YMM4 |Self::YMM5 |Self::YMM6 |Self::YMM7  |
            Self::YMM8 |Self::YMM9 |Self::YMM10|Self::YMM11 |
            Self::YMM12|Self::YMM13|Self::YMM14|Self::YMM15 => Size::Yword,

        }
    }
    pub fn needs_rex(&self) -> bool{
        match self {
            Self::CR8  |
            Self::R8   |Self::R9   |Self::R10  |Self::R11  |
            Self::R12  |Self::R13  |Self::R14  |Self::R15  |
            Self::R8B  |Self::R9B  |Self::R10B |Self::R11B |
            Self::R12B |Self::R13B |Self::R14B |Self::R15B |
            Self::R8W  |Self::R9W  |Self::R10W |Self::R11W |
            Self::R12W |Self::R13W |Self::R14W |Self::R15W |
            Self::R8D  |Self::R9D  |Self::R10D |Self::R11D |
            Self::R12D |Self::R13D |Self::R14D |Self::R15D |
            Self::XMM8 |Self::XMM9 |Self::XMM10|Self::XMM11|
            Self::XMM12|Self::XMM13|Self::XMM14|Self::XMM15|
            Self::YMM8 |Self::YMM9 |Self::YMM10|Self::YMM11|
            Self::SIL  |Self::DIL  |Self::BPL  |Self::SPL  |
            Self::YMM12|Self::YMM13|Self::YMM14|Self::YMM15 => true,
            _ => false
        }
    }
    pub fn to_byte(&self) -> u8{
        match &self {
             Self::R8 |Self::R8B |Self::R8W
            |Self::R8D|Self::XMM8|Self::YMM8
            |Self::AL |Self::AX  |Self::EAX
            |Self::RAX|Self::XMM0|Self::YMM0 => 0,

            Self::R9   | Self::R9B | Self::R9W | Self::R9D |
            Self::CL   | Self::CX  | Self::ECX | Self::RCX |
            Self::XMM1 | Self::YMM1| Self::XMM9|
            Self::YMM9 => 1,

            Self::R10 | Self::R10B | Self::R10W | Self::R10D |
            Self::DL  | Self::DX   | Self::EDX  | Self::XMM2 |
            Self::RDX |
            Self::YMM2| Self::XMM10| Self::YMM10 => 0b10,

            Self::R11 | Self::R11B | Self::R11W | Self::R11D |
            Self::BL  | Self::BX   | Self::EBX  | Self::XMM3 |
            Self::RBX |
            Self::YMM3| Self::XMM11| Self::YMM11 => 0b11,
            
            Self::R12 | Self::R12B | Self::R12W | Self::R12D |
            Self::AH  | Self::SP   | Self::ESP  | Self::XMM4 |
            Self::SPL | Self::RSP  |
            Self::YMM4| Self::XMM12| Self::YMM12 => 0b100,
            
            Self::R13 | Self::R13B | Self::R13W | Self::R13D |
            Self::CH  | Self::BP   | Self::EBP  | Self::XMM5 |
            Self::BPL | Self::RBP  |
            Self::YMM5| Self::XMM13| Self::YMM13 => 0b101,
            
            Self::R14 | Self::R14B | Self::R14W | Self::R14D |
            Self::DH  | Self::SI   | Self::ESI  | Self::XMM6 |
            Self::SIL | Self::RSI  |
            Self::YMM6| Self::XMM14| Self::YMM14 => 0b110,
            
            Self::R15 | Self::R15B | Self::R15W | Self::R15D |
            Self::BH  | Self::DI   | Self::EDI  | Self::XMM7 |
            Self::DIL | Self::RDI  |
            Self::YMM7| Self::XMM15| Self::YMM15 => 0b111,
            _ => 0,
        }
    }
    pub fn variant_of_ax(&self) -> bool{
        match self{
            Self::EAX|Self::AX|Self::AL|Self::RAX => true,
            _ => false
        }
    }
}
