//  rasmx86_64  - reg.rs
//  --------------------
//  made by matissoss
//  licensed under MPL 2.0

use crate::{
    conf::FAST_MODE,
    shr::ast::{
        ToAsmType,
        AsmType
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
    //  AVX (512-bit)
    ZMM0, ZMM1 , ZMM2 , ZMM3 , 
    ZMM4, ZMM5 , ZMM6 , ZMM7 , 
    ZMM8, ZMM9 , ZMM10, ZMM11,
    ZMM12,ZMM13, ZMM14, ZMM15,
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
                    'z' => {
                        match byte_str[3] as char {
                            '0' => reg_ie(str,"zmm0", Register::ZMM0),
                            '1' => reg_ie(str,"zmm1", Register::ZMM1),
                            '2' => reg_ie(str,"zmm2", Register::ZMM2),
                            '3' => reg_ie(str,"zmm3", Register::ZMM3),
                            '4' => reg_ie(str,"zmm4", Register::ZMM4),
                            '5' => reg_ie(str,"zmm5", Register::ZMM5),
                            '6' => reg_ie(str,"zmm6", Register::ZMM6),
                            '7' => reg_ie(str,"zmm7", Register::ZMM7),
                            '8' => reg_ie(str,"zmm8", Register::ZMM8),
                            '9' => reg_ie(str,"zmm9", Register::ZMM9),
                            _   => return Err(())
                        }
                    },
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
                    'z' => {
                        match byte_str[4] as char {
                            '0' => reg_ie(str,"zmm10", Register::ZMM10),
                            '1' => reg_ie(str,"zmm11", Register::ZMM11),
                            '2' => reg_ie(str,"zmm12", Register::ZMM12),
                            '3' => reg_ie(str,"zmm13", Register::ZMM13),
                            '4' => reg_ie(str,"zmm14", Register::ZMM14),
                            '5' => reg_ie(str,"zmm15", Register::ZMM15),
                            _ => Err(())
                        }
                    }
                    _ => Err(())
                }
            }
            _ => Err(())
        };
    }
}

#[allow(unused)]
impl Register{
    pub fn is_16bit(&self) -> bool {
        return match self {
            Self::AX  |Self::BX  |Self::CX  |Self::DX
           |Self::SP  |Self::BP  |Self::SI  |Self::DI
           |Self::R8W |Self::R9W |Self::R10W|Self::R11W
           |Self::R12W|Self::R13W|Self::R14W|Self::R15W => true,
           _ => false
        };
    }
    pub fn is_8bit(&self) -> bool {
        return match self{
            Self::AL  |Self::BL  |Self::CL  |Self::DL  |
            Self::SPL |Self::BPL |Self::SIL |Self::DIL |
            Self::R8B |Self::R9B |Self::R10B|Self::R11B|
            Self::AH  |Self::BH  |Self::CH  |Self::DH  |
            Self::R12B|Self::R13B|Self::R14B|Self::R15B| Self::IP
            => true,
            _ => false
        };
    }
    pub fn is_32bit(&self) -> bool {
        return match self {
            Self::EAX |Self::EBX |Self::ECX |Self::EDX |
            Self::ESP |Self::EBP |Self::ESI |Self::EDI |
            Self::R8D |Self::R9D |Self::R10D|Self::R11D|
            Self::R12D|Self::R13D|Self::R14D|Self::R15D| Self::EIP 
            => true,
            _ => false,
        } 
    }
    pub fn is_64bit(&self) -> bool {
        return !self.is_32bit() 
            && !self.is_16bit() 
            && !self.is_8bit ();
    }
}

impl ToString for Register{
    fn to_string(&self) -> String {
        match self{
            Self::AL   => String::from("al"),
            Self::BL   => String::from("bl"),
            Self::CL   => String::from("cl"),
            Self::DL   => String::from("dl"),
            Self::IP   => String::from("ip"),

            Self::AH   => String::from("ah"),
            Self::BH   => String::from("bh"),
            Self::CH   => String::from("ch"),
            Self::DH   => String::from("dh"),

            Self::SIL  => String::from("sil"),
            Self::DIL  => String::from("dil"),
            Self::SPL  => String::from("spl"),
            Self::BPL  => String::from("bpl"),
            
            Self::SP   => String::from("sp"),
            Self::BP   => String::from("bp"),
            
            Self::R8D  => String::from("r8d"),
            Self::R9D  => String::from("r9d"),
            Self::R10D => String::from("r10d"),
            Self::R11D => String::from("r11d"),
            Self::R12D => String::from("r12d"),
            Self::R13D => String::from("r13d"),
            Self::R14D => String::from("r14d"),
            Self::R15D => String::from("r15d"),

            Self::R8W  => String::from("r8b"),
            Self::R9W  => String::from("r9b"),
            Self::R10W => String::from("r10b"),
            Self::R11W => String::from("r11b"),
            Self::R12W => String::from("r12b"),
            Self::R13W => String::from("r13b"),
            Self::R14W => String::from("r14b"),
            Self::R15W => String::from("r15b"),
            
            Self::AX   => String::from("ax"),
            Self::BX   => String::from("bx"),
            Self::CX   => String::from("cx"),
            Self::DX   => String::from("dx"),
            Self::SI   => String::from("si"),
            Self::DI   => String::from("di"),
            Self::R8B  => String::from("r8b"),
            Self::R9B  => String::from("r9b"),
            Self::R10B => String::from("r10b"),
            Self::R11B => String::from("r11b"),
            Self::R12B => String::from("r12b"),
            Self::R13B => String::from("r13b"),
            Self::R14B => String::from("r14b"),
            Self::R15B => String::from("r15b"),
            
            Self::RAX => String::from("rax"),
            Self::RBX => String::from("rbx"),
            Self::RCX => String::from("rcx"),
            Self::RDX => String::from("rdx"),
            Self::RSI => String::from("rsi"),
            Self::RDI => String::from("rdi"),
            Self::RSP => String::from("rsp"),
            Self::RBP => String::from("rbp"),
            Self::RIP => String::from("rip"),
            Self::R8  => String::from("r8" ),
            Self::R9  => String::from("r9" ),
            Self::R10 => String::from("r10"),
            Self::R11 => String::from("r11"),
            Self::R12 => String::from("r12"),
            Self::R13 => String::from("r13"),
            Self::R14 => String::from("r14"),
            Self::R15 => String::from("r15"),
            
            Self::EAX => String::from("eax"),
            Self::EBX => String::from("ebx"),
            Self::ECX => String::from("ecx"),
            Self::EDX => String::from("edx"),
            Self::ESI => String::from("esi"),
            Self::EDI => String::from("edi"),
            Self::ESP => String::from("esp"),
            Self::EIP => String::from("eip"),
            Self::EBP => String::from("ebp"),

            Self::CR0 => String::from("cr0"),
            Self::CR2 => String::from("cr2"),
            Self::CR3 => String::from("cr3"),
            Self::CR4 => String::from("cr4"),
            Self::CR8 => String::from("cr8"),
            
            Self::DR0 => String::from("dr0"),
            Self::DR1 => String::from("dr1"),
            Self::DR2 => String::from("dr2"),
            Self::DR3 => String::from("dr3"),
            Self::DR6 => String::from("dr6"),
            Self::DR7 => String::from("dr7"),

            Self::XMM0 => String::from("xmm0"),
            Self::XMM1 => String::from("xmm1"),
            Self::XMM2 => String::from("xmm2"),
            Self::XMM3 => String::from("xmm3"),
            Self::XMM4 => String::from("xmm4"),
            Self::XMM5 => String::from("xmm5"),
            Self::XMM6 => String::from("xmm6"),
            Self::XMM7 => String::from("xmm7"),
            Self::XMM8 => String::from("xmm8"),
            Self::XMM9 => String::from("xmm9"),
            Self::XMM10 => String::from("xmm10"),
            Self::XMM11 => String::from("xmm11"),
            Self::XMM12 => String::from("xmm12"),
            Self::XMM13 => String::from("xmm13"),
            Self::XMM14 => String::from("xmm14"),
            Self::XMM15 => String::from("xmm15"),
            
            Self::YMM0 => String::from("ymm0"),
            Self::YMM1 => String::from("ymm1"),
            Self::YMM2 => String::from("ymm2"),
            Self::YMM3 => String::from("ymm3"),
            Self::YMM4 => String::from("ymm4"),
            Self::YMM5 => String::from("ymm5"),
            Self::YMM6 => String::from("ymm6"),
            Self::YMM7 => String::from("ymm7"),
            Self::YMM8 => String::from("ymm8"),
            Self::YMM9 => String::from("ymm9"),
            Self::YMM10 => String::from("ymm10"),
            Self::YMM11 => String::from("ymm11"),
            Self::YMM12 => String::from("ymm12"),
            Self::YMM13 => String::from("ymm13"),
            Self::YMM14 => String::from("ymm14"),
            Self::YMM15 => String::from("ymm15"),
            
            Self::ZMM0 => String::from("zmm0"),
            Self::ZMM1 => String::from("zmm1"),
            Self::ZMM2 => String::from("zmm2"),
            Self::ZMM3 => String::from("zmm3"),
            Self::ZMM4 => String::from("zmm4"),
            Self::ZMM5 => String::from("zmm5"),
            Self::ZMM6 => String::from("zmm6"),
            Self::ZMM7 => String::from("zmm7"),
            Self::ZMM8 => String::from("zmm8"),
            Self::ZMM9 => String::from("zmm9"),
            Self::ZMM10 => String::from("zmm10"),
            Self::ZMM11 => String::from("zmm11"),
            Self::ZMM12 => String::from("zmm12"),
            Self::ZMM13 => String::from("zmm13"),
            Self::ZMM14 => String::from("zmm14"),
            Self::ZMM15 => String::from("zmm15"),
        }
    }
}

impl ToAsmType for Register{
    fn asm_type(&self) -> AsmType{
        return AsmType::Reg;
    }
}
