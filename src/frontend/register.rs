//  rasmx86_64  -   register.rs
//  ---------------------------
//  made by matissoss
//  licensed under MPL 2.0

use std::str::FromStr;

#[derive(Debug)]
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
    //XMM0 , XMM1 , XMM2 , XMM3 , 
    //XMM4 , XMM5 , XMM6 , XMM7 , 
    //XMM8 , XMM9 , XMM10, XMM11,
    //XMM12, XMM13, XMM14, XMM15,

    //  AVX
    //YMM0, YMM1 , YMM2 , YMM3 , 
    //YMM4, YMM5 , YMM6 , YMM7 , 
    //YMM8, YMM9 , YMM10, YMM11,
    //YMM12,YMM13, YMM14, YMM15,
    //  AVX (512-bit)
    //ZMM0, ZMM1 , ZMM2 , ZMM3 , 
    //ZMM4, ZMM5 , ZMM6 , ZMM7 , 
    //ZMM8, ZMM9 , ZMM10, ZMM11,
    //ZMM12,ZMM13, ZMM14, ZMM15,
}

pub enum RegErr{
    TooShort    (usize),
    TooLong     (usize),
    Unknown     (String),
    Unsupported (String),
}

impl ToString for RegErr{
    fn to_string(&self) -> String {
        return match self{
            Self::TooShort(reg_len)     => format!("E-SYN-0001: Register is too short! LEN = {}", reg_len),
            Self::TooLong (reg_len)     => format!("E-SYN-0002: Register is too long ! LEN = {}", reg_len),
            Self::Unknown (reg_str)     => format!("E-SYN-0003: Unknown Register: `{}`", reg_str),
            Self::Unsupported(reg_str)  => format!("E-SYN-0004: Unsupported Register: `{}`", reg_str),
        }
    }
}

impl FromStr for Register{
    type Err = RegErr;
    fn from_str(str: &str) -> Result<Self, <Self as FromStr>::Err>{
        let byte_str = str.as_bytes();
        return match byte_str.len(){
            1 => Err(RegErr::TooShort(str.len())),
            2 => {
                match byte_str[1] as char{
                    'i' => {
                        match byte_str[0] as char {
                            's' => return Ok(Register::SI),
                            'd' => return Ok(Register::DI),
                            _   => return Err(RegErr::Unknown(str.to_string()))
                        }
                    },
                    'l' => {
                        match byte_str[0] as char {
                            'a' => return Ok(Register::AL),
                            'b' => return Ok(Register::BL),
                            'c' => return Ok(Register::CL),
                            'd' => return Ok(Register::DL),
                            _   => return Err(RegErr::Unknown(str.to_string()))
                        }
                    }
                    'x' => {
                        match byte_str[0] as char {
                            'a' => return Ok(Register::AX),
                            'b' => return Ok(Register::BX),
                            'c' => return Ok(Register::CX),
                            'd' => return Ok(Register::DX),
                            _   => return Err(RegErr::Unknown(str.to_string()))
                        }
                    }
                    'h' => {
                        match byte_str[0] as char {
                            'a' => return Ok(Register::AH),
                            'b' => return Ok(Register::BH),
                            'c' => return Ok(Register::CH),
                            'd' => return Ok(Register::DH),
                            _   => return Err(RegErr::Unknown(str.to_string()))
                        }
                    },
                    'p' => {
                        match byte_str[0] as char{
                            'i' => return Ok(Register::IP),
                            'b' => return Ok(Register::BP),
                            's' => return Ok(Register::SP),
                            _   => return Err(RegErr::Unknown(str.to_string()))
                        }
                    },
                    '8' => return Ok(Register::R8),
                    '9' => return Ok(Register::R9),
                    _ => Err(RegErr::Unknown(str.to_string()))
                }
            },
            // prev = 2; byte_str.len()
            3 => {
                match byte_str[0] as char {
                    'r' => {
                        match byte_str[1] as char{
                            'a' => return Ok(Register::RAX),
                            'b' => {
                                match byte_str[2] as char {
                                    'x' => return Ok(Register::RBX),
                                    'p' => return Ok(Register::RBP),
                                    _ => Err(RegErr::Unknown(str.to_string()))
                                }
                            },
                            'c' => return Ok(Register::RCX),
                            'd' => {
                                match byte_str[2] as char {
                                    'x' => return Ok(Register::RDX),
                                    'd' => return Ok(Register::RDI),
                                    _ => Err(RegErr::Unknown(str.to_string()))
                                }
                            },
                            's' => {
                                match byte_str[2] as char {
                                    'i' => return Ok(Register::RSI),
                                    'p' => return Ok(Register::RSP),
                                    _ => Err(RegErr::Unknown(str.to_string()))
                                }
                            }
                            '1' => {
                                match byte_str[2] as char {
                                    '0' => return Ok(Register::R10),
                                    '1' => return Ok(Register::R11),
                                    '2' => return Ok(Register::R12),
                                    '3' => return Ok(Register::R13),
                                    '4' => return Ok(Register::R14),
                                    '5' => return Ok(Register::R15),
                                    _   => Err(RegErr::Unknown(str.to_string()))
                                }
                            }
                            '8' => {
                                match byte_str[2] as char {
                                    'b' => return Ok(Register::R8B),
                                    'w' => return Ok(Register::R8W),
                                    'd' => return Ok(Register::R8D),
                                    _   => Err(RegErr::Unknown(str.to_string()))
                                }
                            }
                            '9' => {
                                match byte_str[2] as char {
                                    'b' => return Ok(Register::R9B),
                                    'w' => return Ok(Register::R9W),
                                    'd' => return Ok(Register::R9D),
                                    _   => Err(RegErr::Unknown(str.to_string()))
                                }
                            }
                            'i' => return Ok(Register::RIP),
                            _   => Err(RegErr::Unknown(str.to_string()))
                        }
                    },
                    // prev = 'r'; byte_str[0]
                    'e' => {
                        match byte_str[1] as char {
                            'a' => return Ok(Register::EAX),
                            'b' => {
                                match byte_str[2] as char {
                                    'p' => return Ok(Register::EBP),
                                    'x' => return Ok(Register::EBX),
                                    _   => Err(RegErr::Unknown(str.to_string()))
                                }
                            },
                            'c' => return Ok(Register::ECX),
                            'd' => {
                                match byte_str[2] as char {
                                    'i' => return Ok(Register::EDI),
                                    'x' => return Ok(Register::EDX),
                                    _   => Err(RegErr::Unknown(str.to_string()))
                                }
                            }
                            's' => {
                                match byte_str[2] as char {
                                    'i' => return Ok(Register::ESI),
                                    'p' => return Ok(Register::ESP),
                                    _   => Err(RegErr::Unknown(str.to_string()))
                                }
                            }
                            'i' => return Ok(Register::EIP),
                            _   => Err(RegErr::Unknown(str.to_string()))
                        }
                    }
                    // prev = 'e'; byte_str[0]
                    's' => {
                        match byte_str[1] as char {
                            'p' => return Ok(Register::SPL),
                            'i' => return Ok(Register::SIL),
                            _   => Err(RegErr::Unknown(str.to_string()))
                        }
                    }
                    'b' => return Ok(Register::BPL),
                    'c' => {
                        match byte_str[2] as char {
                            '0' => return Ok(Register::CR0),
                            '2' => return Ok(Register::CR2),
                            '3' => return Ok(Register::CR3),
                            '4' => return Ok(Register::CR4),
                            '8' => return Ok(Register::CR8),
                            _   => Err(RegErr::Unknown(str.to_string()))
                        }
                    }
                    'd' => {
                        match byte_str[2] as char{
                            'l' => return Ok(Register::DIL),
                            '0' => return Ok(Register::DR0),
                            '1' => return Ok(Register::DR1),
                            '2' => return Ok(Register::DR2),
                            '3' => return Ok(Register::DR3),
                            '6' => return Ok(Register::DR6),
                            '7' => return Ok(Register::DR7),
                            _   => Err(RegErr::Unknown(str.to_string()))
                        }
                    },
                    _   => Err(RegErr::Unknown(str.to_string()))
                }
            }
            // prev = 3; byte_str.len()
            4 => {
                match byte_str[0] as char {
                    'r' => {
                        match byte_str[2] as char{
                            '0' => {
                                match byte_str[3] as char {
                                    'd' => return Ok(Register::R10D),
                                    'b' => return Ok(Register::R10B),
                                    'w' => return Ok(Register::R10W),
                                    _   => Err(RegErr::Unknown(str.to_string()))
                                }
                            },
                            '1' => {
                                match byte_str[3] as char {
                                    'd' => return Ok(Register::R11D),
                                    'b' => return Ok(Register::R11B),
                                    'w' => return Ok(Register::R11W),
                                    _   => Err(RegErr::Unknown(str.to_string()))
                                }
                            },
                            '2' => {
                                match byte_str[3] as char {
                                    'd' => return Ok(Register::R12D),
                                    'b' => return Ok(Register::R12B),
                                    'w' => return Ok(Register::R12W),
                                    _   => Err(RegErr::Unknown(str.to_string()))
                                }
                            },
                            '3' => {
                                match byte_str[3] as char {
                                    'd' => return Ok(Register::R13D),
                                    'b' => return Ok(Register::R13B),
                                    'w' => return Ok(Register::R13W),
                                    _   => Err(RegErr::Unknown(str.to_string()))
                                }
                            },
                            '4' => {
                                match byte_str[3] as char {
                                    'd' => return Ok(Register::R14D),
                                    'b' => return Ok(Register::R14B),
                                    'w' => return Ok(Register::R14W),
                                    _   => Err(RegErr::Unknown(str.to_string()))
                                }
                            },
                            '5' => {
                                match byte_str[3] as char {
                                    'd' => return Ok(Register::R15D),
                                    'b' => return Ok(Register::R15B),
                                    'w' => return Ok(Register::R15W),
                                    _   => Err(RegErr::Unknown(str.to_string()))
                                }
                            },
                            _   => Err(RegErr::Unknown(str.to_string()))
                        }
                    }
                    'x' => {
                        match byte_str[3] as char {
                            '0' => return Err(RegErr::Unsupported("xmm0".to_string())), // Ok(Register::XMM0),
                            '1' => return Err(RegErr::Unsupported("xmm1".to_string())), // Ok(Register::XMM1),
                            '2' => return Err(RegErr::Unsupported("xmm2".to_string())), // Ok(Register::XMM2),
                            '3' => return Err(RegErr::Unsupported("xmm3".to_string())), // Ok(Register::XMM3),
                            '4' => return Err(RegErr::Unsupported("xmm4".to_string())), // Ok(Register::XMM4),
                            '5' => return Err(RegErr::Unsupported("xmm5".to_string())), // Ok(Register::XMM5),
                            '6' => return Err(RegErr::Unsupported("xmm6".to_string())), // Ok(Register::XMM6),
                            '7' => return Err(RegErr::Unsupported("xmm7".to_string())), // Ok(Register::XMM7),
                            '8' => return Err(RegErr::Unsupported("xmm8".to_string())), // Ok(Register::XMM8),
                            '9' => return Err(RegErr::Unsupported("xmm9".to_string())), // Ok(Register::XMM9),
                            _   => return Err(RegErr::Unknown(str.to_string()))
                        }
                    }
                    'y' => {
                        match byte_str[3] as char {
                            '0' => return Err(RegErr::Unsupported("ymm0".to_string())), // Ok(Register::YMM0),
                            '1' => return Err(RegErr::Unsupported("ymm1".to_string())), // Ok(Register::YMM1),
                            '2' => return Err(RegErr::Unsupported("ymm2".to_string())), // Ok(Register::YMM2),
                            '3' => return Err(RegErr::Unsupported("ymm3".to_string())), // Ok(Register::YMM3),
                            '4' => return Err(RegErr::Unsupported("ymm4".to_string())), // Ok(Register::YMM4),
                            '5' => return Err(RegErr::Unsupported("ymm5".to_string())), // Ok(Register::YMM5),
                            '6' => return Err(RegErr::Unsupported("ymm6".to_string())), // Ok(Register::YMM6),
                            '7' => return Err(RegErr::Unsupported("ymm7".to_string())), // Ok(Register::YMM7),
                            '8' => return Err(RegErr::Unsupported("ymm8".to_string())), // Ok(Register::YMM8),
                            '9' => return Err(RegErr::Unsupported("ymm9".to_string())), // Ok(Register::YMM9),
                            _   => return Err(RegErr::Unknown(str.to_string()))
                        }
                    }
                    'z' => {
                        match byte_str[3] as char {
                            '0' => return Err(RegErr::Unsupported("zmm0".to_string())), // Ok(Register::ZMM0),
                            '1' => return Err(RegErr::Unsupported("zmm1".to_string())), // Ok(Register::ZMM1),
                            '2' => return Err(RegErr::Unsupported("zmm2".to_string())), // Ok(Register::ZMM2),
                            '3' => return Err(RegErr::Unsupported("zmm3".to_string())), // Ok(Register::ZMM3),
                            '4' => return Err(RegErr::Unsupported("zmm4".to_string())), // Ok(Register::ZMM4),
                            '5' => return Err(RegErr::Unsupported("zmm5".to_string())), // Ok(Register::ZMM5),
                            '6' => return Err(RegErr::Unsupported("zmm6".to_string())), // Ok(Register::ZMM6),
                            '7' => return Err(RegErr::Unsupported("zmm7".to_string())), // Ok(Register::ZMM7),
                            '8' => return Err(RegErr::Unsupported("zmm8".to_string())), // Ok(Register::ZMM8),
                            '9' => return Err(RegErr::Unsupported("zmm9".to_string())), // Ok(Register::ZMM9),
                            _   => return Err(RegErr::Unknown(str.to_string()))
                        }
                    },
                    _   => return Err(RegErr::Unknown(str.to_string()))
                }
            },
            5 => {
                match byte_str[0] as char {
                    'x' => {
                        match byte_str[4] as char {
                            '0' => return Err(RegErr::Unsupported("xmm10".to_string())), // Ok(Register::XMM10),
                            '1' => return Err(RegErr::Unsupported("xmm11".to_string())), // Ok(Register::XMM11),
                            '2' => return Err(RegErr::Unsupported("xmm12".to_string())), // Ok(Register::XMM12),
                            '3' => return Err(RegErr::Unsupported("xmm13".to_string())), // Ok(Register::XMM13),
                            '4' => return Err(RegErr::Unsupported("xmm14".to_string())), // Ok(Register::XMM14),
                            '5' => return Err(RegErr::Unsupported("xmm15".to_string())), // Ok(Register::XMM15),
                            _   => return Err(RegErr::Unknown(str.to_string()))
                        }
                    }
                    'y' => {
                        match byte_str[4] as char {
                            '0' => return Err(RegErr::Unsupported("xmm10".to_string())), // Ok(Register::YMM10),
                            '1' => return Err(RegErr::Unsupported("xmm11".to_string())), // Ok(Register::YMM11),
                            '2' => return Err(RegErr::Unsupported("xmm12".to_string())), // Ok(Register::YMM12),
                            '3' => return Err(RegErr::Unsupported("xmm13".to_string())), // Ok(Register::YMM13),
                            '4' => return Err(RegErr::Unsupported("xmm14".to_string())), // Ok(Register::YMM14),
                            '5' => return Err(RegErr::Unsupported("xmm15".to_string())), // Ok(Register::YMM15),
                            _   => return Err(RegErr::Unknown(str.to_string()))
                        }
                    },
                    'z' => {
                        match byte_str[4] as char {
                            '0' => return Err(RegErr::Unsupported("zmm10".to_string())), // Ok(Register::ZMM10),
                            '1' => return Err(RegErr::Unsupported("zmm11".to_string())), // Ok(Register::ZMM11),
                            '2' => return Err(RegErr::Unsupported("zmm12".to_string())), // Ok(Register::ZMM12),
                            '3' => return Err(RegErr::Unsupported("zmm13".to_string())), // Ok(Register::ZMM13),
                            '4' => return Err(RegErr::Unsupported("zmm14".to_string())), // Ok(Register::ZMM14),
                            '5' => return Err(RegErr::Unsupported("zmm15".to_string())), // Ok(Register::ZMM15),
                            _   => return Err(RegErr::Unknown(str.to_string()))
                        }
                    }
                    _   => return Err(RegErr::Unknown(str.to_string()))
                }
            }
            _ => Err(RegErr::TooLong(str.len()))
        };
    }
}
