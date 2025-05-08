// rasmx86_64 - src/shr/ins.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::conf::FAST_MODE;
use crate::shr::size::Size;
use std::str::FromStr;

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mnemonic {
    MOV  , ADD , SUB ,
    IMUL , MUL , DIV ,
    IDIV , AND , OR  ,
    NOT  , NEG , XOR ,
    SHR  , SAR , SHL ,
    SAL  , LEA , INC ,
    DEC  , CMP , TEST,
    
    JMP, CALL,

    JE , JZ , JNZ,
    JNE, JL , JLE,
    JG , JGE,

    SYSCALL, RET,
    NOP,

    POP   , POPF  , POPFD,
    POPFQ , PUSH  , PUSHF,
    PUSHFD, PUSHFQ,

    CPUID,

    // SSE extension
    ADDPS  , ADDSS,
    SUBPS  , SUBSS,
    MULPS  , MULSS,
    DIVPS  , DIVSS,
    RCPPS  , RCPSS,
    SQRTPS , SQRTSS,
    RSQRTPS, RSQRTSS,
    MINPS  , MINSS,
    MAXPS  , MAXSS,

    ORPS   , ANDPS,
    ANDNPS , XORPS,
    CMPPS  , CMPSS,

    COMISS , UCOMISS,

    CVTSI2SS, 
    CVTPI2PS,
    CVTPS2PI,
    CVTSS2SI,
    CVTTPS2PI,

    SHUFPS, UNPCKLPS, UNPCKHPS,

    MOVAPS, MOVUPS, MOVSS,
    MOVLPS, MOVHPS,
    
    MOVLHPS,MOVHLPS,
    
    // SSE2 extension
    ADDPD  , ADDSD,
    SUBPD  , SUBSD,
    MULPD  , MULSD,
    DIVPD  , DIVSD,
    SQRTPD , SQRTSD,
    MINPD  , MINSD,
    MAXPD  , MAXSD,
    ORPD   , ANDPD,
    ANDNPD , XORPD,

    CMPPD  , CMPSD,
    COMISD , UCOMISD,

    MOVAPD , MOVUPD,
    MOVHPD , MOVLPD,
    MOVSD  , MOVMSKPD,

    MOVDQA , MOVQ2DQ,
    MOVDQ2Q,

    CVTPD2PI, CVTTPD2PI,

    CVTPI2PD, CVTDQ2PD,
    CVTSD2SI, CVTTSD2SI,
    CVTSI2SD, CVTPS2DQ,
    CVTTPS2DQ, CVTDQ2PS,

    CVTPS2PD, CVTPD2PS,
    CVTSS2SD, CVTPS2SS,

    PSUBQ,
    PSHUFD,
    PSLLDQ,
    PSRLDQ,
    PMULUDQ,
    PSHUFLW,
    PSHUFHW,
    PUNPCKHQDQ,
    PUNPCKLQDQ,

    MASKMOVDQU,

    MFENCE, LFENCE,
    CLFLUSH, PAUSE,
    MOVNTPD, 
    MOVNTDQ,
    MOVNTI,

    // SSE3 extension
    ADDSUBPS,
    ADDSUBPD,

    HADDPS, HSUBPS,
    HADDPD, HSUBPD,
    MOVSLDUP, MOVSHDUP,
    MOVDDUP, LDDQU,
    MONITOR, MWAIT,

    // SSSE3 extension

    PABSW, 
    PABSD, 
    PABSB,
    PSIGNW, 
    PSIGND, 
    PSIGNB,
    PHSUBW,
    PHSUBD,
    PHADDW,
    PHADDD,
    PSHUFB,
    PHSUBSW,
    PHADDSW,
    PALIGNR,
    PMULHRSW,
    PMADDUBSW,

    // MMX extension
    MOVD, MOVQ,
    
    PADDB , PADDW , PADDD  , PADDQ  ,
    PADDSB, PADDSW, PADDUSB, PADDUSW,
    
    PSUBB , PSUBW  , PSUBD  , PSUBSB ,
    PSUBSW, PSUBUSB, PSUBUSW, PANDN  ,

    PMULHW, PMULLW,

    PMADDWD,

    PCMPEQB, PCMPEQW, PCMPEQD,
    PCMPGTB, PCMPGTW, PCMPGTD,
    
    PACKUSWB, PACKSSWB, PACKSSDW,
    
    PUNPCKLBW, PUNPCKLWD, PUNPCKLDQ,
    PUNPCKHBW, PUNPCKHWD, PUNPCKHDQ,

    POR, PAND, PANND, PXOR,
    
    PSLLW, PSLLD, PSLLQ,
    PSRLW, PSRLD, PSRLQ,
    PSRAW, PSRAD,

    EMMS,

    // SSE4_1 and SSE4_2
    DPPS, DPPD,
    
    PMAXD, PMIND, PTEST, CRC32,
    
    PEXTRB, PEXTRW, PEXTRD, PEXTRQ,
    PINSRB, PINSRD, PINSRQ, PMAXSB,
    PMAXSD, PMAXUB, PMINSB, PMINSD,
    PMINUB, PMULDQ, PMULLD, POPCNT,
    
    BLENDPS, BLENDPD, PBLENDW, PCMPEQQ,
    ROUNDPD, ROUNDPS, ROUNDSD, ROUNDSS,
    MPSADBW, PCMPGTQ,
    
    BLENDVPS, BLENDVPD, PBLENDVB, INSERTPS,
    PMOVSXBW, PMOVSXBD, PMOVSXBQ, PMOVSXWD,
    PMOVSXDQ, PMOVZXBW, PMOVZXBD, PMOVZXBQ,
    PMOVZXWD, PMOVZXDQ, PACKUSDW, PCMPESTRI,
    MOVNTDQA,

    EXTRACTPS, PCMPESTRM, PCMPISTRI, PCMPISTRM,
    
    PHMINPOSUW,
}

#[inline(always)]
fn ins_ie(i: &str, c: &str, ins: Mnemonic) -> Result<Mnemonic, ()> {
    if FAST_MODE {
        Ok(ins)
    } else {
        if i == c {
            Ok(ins)
        } else {
            Err(())
        }
    }
}

impl FromStr for Mnemonic {
    type Err = ();
    fn from_str(str_ins: &str) -> Result<Self, <Self as FromStr>::Err> {
        if let Some(m) = mnem_fromstr(str_ins) {
            Ok(m)
        } else {
            Err(())
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Mnemonic {
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}

impl Mnemonic {
    pub fn allows_diff_size(&self, _left: Option<Size>, _right: Option<Size>) -> bool {
        false
    }
    pub fn allows_mem_mem(&self) -> bool {
        false
    }
    pub fn defaults_to_64bit(&self) -> bool {
        matches!(
            self,
            Self::PUSH | Self::POP | Self::PADDB | Self::PADDW | Self::PADDD | Self::PADDQ
        )
    }
}

#[inline(always)]
fn s<T>(i: T) -> Option<T> {
    Some(i)
}

#[inline(always)]
fn n<T>() -> Option<T> {
    None
}

type Ins = Mnemonic;
pub fn mnem_fromstr(str: &str) -> Option<Ins> {
    let rstr = str.chars().collect::<Vec<char>>();
    match str.len() {
        0 | 1 => n(),
        2 => match rstr[0] {
            'j' => match rstr[1] {
                'e' => s(Ins::JE),
                'z' => s(Ins::JZ),
                'l' => s(Ins::JL),
                'g' => s(Ins::JG),
                _ => n(),
            },
            'o' => match rstr[1] {
                'r' => s(Ins::OR),
                _ => n(),
            },
            _ => n(),
        },
        3 => match rstr[0] {
            'l' => match rstr[1] {
                'e' => match rstr[2] {
                    'a' => s(Ins::LEA),
                    _ => n(),
                },
                _ => n(),
            },
            'r' => match rstr[1] {
                'e' => match rstr[2] {
                    't' => s(Ins::RET),
                    _ => n(),
                },
                _ => n(),
            },
            'j' => match rstr[1] {
                'n' => match rstr[2] {
                    'e' => s(Ins::JNE),
                    'z' => s(Ins::JNZ),
                    _ => n(),
                },
                'g' => match rstr[2] {
                    'e' => s(Ins::JGE),
                    _ => n(),
                },
                'l' => match rstr[2] {
                    'e' => s(Ins::JLE),
                    _ => n(),
                },
                'm' => match rstr[2] {
                    'p' => s(Ins::JMP),
                    _ => n(),
                },
                _ => n(),
            },
            'n' => match rstr[1] {
                'e' => match rstr[2] {
                    'g' => s(Ins::NEG),
                    _ => n(),
                },
                'o' => match rstr[2] {
                    'p' => s(Ins::NOP),
                    't' => s(Ins::NOT),
                    _ => n(),
                },
                _ => n(),
            },
            'p' => match rstr[1] {
                'o' => match rstr[2] {
                    'p' => s(Ins::POP),
                    'r' => s(Ins::POR),
                    _ => n(),
                },
                _ => n(),
            },
            'a' => match rstr[1] {
                'n' => match rstr[2] {
                    'd' => s(Ins::AND),
                    _ => n(),
                },
                'd' => match rstr[2] {
                    'd' => s(Ins::ADD),
                    _ => n(),
                },
                _ => n(),
            },
            'm' => match rstr[1] {
                'o' => match rstr[2] {
                    'v' => s(Ins::MOV),
                    _ => n(),
                },
                'u' => match rstr[2] {
                    'l' => s(Ins::MUL),
                    _ => n(),
                },
                _ => n(),
            },
            's' => match rstr[1] {
                'h' => match rstr[2] {
                    'l' => s(Ins::SHL),
                    'r' => s(Ins::SHR),
                    _ => n(),
                },
                'a' => match rstr[2] {
                    'l' => s(Ins::SAL),
                    'r' => s(Ins::SAR),
                    _ => n(),
                },
                'u' => match rstr[2] {
                    'b' => s(Ins::SUB),
                    _ => n(),
                },
                _ => n(),
            },
            'd' => match rstr[1] {
                'e' => match rstr[2] {
                    'c' => s(Ins::DEC),
                    _ => n(),
                },
                'i' => match rstr[2] {
                    'v' => s(Ins::DIV),
                    _ => n(),
                },
                _ => n(),
            },
            'x' => match rstr[1] {
                'o' => match rstr[2] {
                    'r' => s(Ins::XOR),
                    _ => n(),
                },
                _ => n(),
            },
            'c' => match rstr[1] {
                'm' => match rstr[2] {
                    'p' => s(Ins::CMP),
                    _ => n(),
                },
                _ => n(),
            },
            'i' => match rstr[1] {
                'n' => match rstr[2] {
                    'c' => s(Ins::INC),
                    _ => n(),
                },
                _ => n(),
            },
            _ => n(),
        },
        4 => match rstr[0] {
            'e' => match rstr[1] {
                'm' => match rstr[2] {
                    'm' => match rstr[3] {
                        's' => s(Ins::EMMS),
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'm' => match rstr[1] {
                'o' => match rstr[2] {
                    'v' => match rstr[3] {
                        'd' => s(Ins::MOVD),
                        'q' => s(Ins::MOVQ),
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'c' => match rstr[1] {
                'a' => match rstr[2] {
                    'l' => match rstr[3] {
                        'l' => s(Ins::CALL),
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            't' => match rstr[1] {
                'e' => match rstr[2] {
                    's' => match rstr[3] {
                        't' => s(Ins::TEST),
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'i' => match rstr[1] {
                'd' => match rstr[2] {
                    'i' => match rstr[3] {
                        'v' => s(Ins::IDIV),
                        _ => n(),
                    },
                    _ => n(),
                },
                'm' => match rstr[2] {
                    'u' => match rstr[3] {
                        'l' => s(Ins::IMUL),
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'p' => match rstr[1] {
                'x' => match rstr[2] {
                    'o' => match rstr[3] {
                        'r' => s(Ins::PXOR),
                        _ => n(),
                    },
                    _ => n(),
                },
                'a' => match rstr[2] {
                    'n' => match rstr[3] {
                        'd' => s(Ins::PAND),
                        _ => n(),
                    },
                    _ => n(),
                },
                'o' => match rstr[2] {
                    'p' => match rstr[3] {
                        'f' => s(Ins::POPF),
                        _ => n(),
                    },
                    _ => n(),
                },
                'u' => match rstr[2] {
                    's' => match rstr[3] {
                        'h' => s(Ins::PUSH),
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'o' => match rstr[1] {
                'r' => match rstr[2] {
                    'p' => match rstr[3] {
                        's' => s(Ins::ORPS),
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'd' => match rstr[1] {
                'p' => match rstr[2] {
                    'p' => match rstr[3] {
                        's' => s(Ins::DPPS),
                        'd' => s(Ins::DPPD),
                        _   => n()
                    }
                    _ => n()
                }
                _ => n()
            }
            _ => n(),
        },
        5 => match rstr[0] {
            'l' => match rstr[1] {
                'd' => match rstr[2] {
                    'd' => match rstr[3] {
                        'q' => match rstr[4] {
                            'u' => s(Ins::LDDQU),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'x' => match rstr[1] {
                'o' => match rstr[2] {
                    'r' => match rstr[3] {
                        'p' => match rstr[4] {
                            'd' => s(Ins::XORPD),
                            's' => s(Ins::XORPS),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'a' => match rstr[1] {
                'n' => match rstr[2] {
                    'd' => match rstr[3] {
                        'p' => match rstr[4] {
                            's' => s(Ins::ANDPS),
                            'd' => s(Ins::ANDPD),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'd' => match rstr[2] {
                    'd' => match rstr[3] {
                        's' => match rstr[4] {
                            's' => s(Ins::ADDSS),
                            'd' => s(Ins::ADDSD),
                            _ => n(),
                        },
                        'p' => match rstr[4] {
                            's' => s(Ins::ADDPS),
                            'd' => s(Ins::ADDPD),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'd' => match rstr[1] {
                'i' => match rstr[2] {
                    'v' => match rstr[3] {
                        'p' => match rstr[4] {
                            'd' => s(Ins::DIVPD),
                            's' => s(Ins::DIVPS),
                            _ => n(),
                        },
                        's' => match rstr[4] {
                            's' => s(Ins::DIVSS),
                            'd' => s(Ins::DIVSD),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'm' => match rstr[1] {
                'o' => match rstr[2] {
                    'v' => match rstr[3] {
                        's' => match rstr[4] {
                            's' => s(Ins::MOVSS),
                            'd' => s(Ins::MOVSD),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'u' => match rstr[2] {
                    'l' => match rstr[3] {
                        'p' => match rstr[4] {
                            'd' => s(Ins::MULPD),
                            's' => s(Ins::MULPS),
                            _ => n(),
                        },
                        's' => match rstr[4] {
                            's' => s(Ins::MULSS),
                            'd' => s(Ins::MULSD),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'i' => match rstr[2] {
                    'n' => match rstr[3] {
                        'p' => match rstr[4] {
                            's' => s(Ins::MINPS),
                            'd' => s(Ins::MINPD),
                            _ => n(),
                        },
                        's' => match rstr[4] {
                            's' => s(Ins::MINSS),
                            'd' => s(Ins::MINSD),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'a' => match rstr[2] {
                    'x' => match rstr[3] {
                        'p' => match rstr[4] {
                            's' => s(Ins::MAXPS),
                            'd' => s(Ins::MAXPD),
                            _ => n(),
                        },
                        's' => match rstr[4] {
                            's' => s(Ins::MAXSS),
                            'd' => s(Ins::MAXSD),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'w' => match rstr[2] {
                    'a' => match rstr[3] {
                        'i' => match rstr[4] {
                            't' => s(Ins::MWAIT),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            's' => match rstr[1] {
                'u' => match rstr[2] {
                    'b' => match rstr[3] {
                        'p' => match rstr[4] {
                            'd' => s(Ins::SUBPD),
                            's' => s(Ins::SUBPS),
                            _ => n(),
                        },
                        's' => match rstr[4] {
                            's' => s(Ins::SUBSS),
                            'd' => s(Ins::SUBSD),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'c' => match rstr[1] {
                'm' => match rstr[2] {
                    'p' => match rstr[3] {
                        'p' => match rstr[4] {
                            'd' => s(Ins::CMPPD),
                            's' => s(Ins::CMPPS),
                            _ => n(),
                        },
                        's' => match rstr[4] {
                            'd' => s(Ins::CMPSD),
                            's' => s(Ins::CMPSS),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'p' => match rstr[2] {
                    'u' => match rstr[3] {
                        'i' => match rstr[4] {
                            'd' => s(Ins::CPUID),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'p' => match rstr[1] {
                'a' => match rstr[2] {
                    'n' => match rstr[3] {
                        'd' => match rstr[4] {
                            'n' => s(Ins::PANDN),
                            _ => n(),
                        },
                        'n' => match rstr[4] {
                            'd' => s(Ins::PANND),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'd' => match rstr[3] {
                        'd' => match rstr[4] {
                            'q' => s(Ins::PADDQ),
                            'd' => s(Ins::PADDD),
                            'w' => s(Ins::PADDW),
                            'b' => s(Ins::PADDB),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'b' => match rstr[3] {
                        's' => match rstr[4] {
                            'd' => s(Ins::PABSD),
                            'w' => s(Ins::PABSW),
                            'b' => s(Ins::PABSB),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'u' => match rstr[3] {
                        's' => match rstr[4] {
                            'e' => s(Ins::PAUSE),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                's' => match rstr[2] {
                    'l' => match rstr[3] {
                        'l' => match rstr[4] {
                            'w' => s(Ins::PSLLW),
                            'd' => s(Ins::PSLLD),
                            'q' => s(Ins::PSLLQ),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'r' => match rstr[3] {
                        'a' => match rstr[4] {
                            'd' => s(Ins::PSRAD),
                            'w' => s(Ins::PSRAW),
                            _ => n(),
                        },
                        'l' => match rstr[4] {
                            'w' => s(Ins::PSRLW),
                            'd' => s(Ins::PSRLD),
                            'q' => s(Ins::PSRLQ),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'u' => match rstr[3] {
                        'b' => match rstr[4] {
                            'q' => s(Ins::PSUBQ),
                            'd' => s(Ins::PSUBD),
                            'w' => s(Ins::PSUBW),
                            'b' => s(Ins::PSUBB),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'o' => match rstr[2] {
                    'p' => match rstr[3] {
                        'f' => match rstr[4] {
                            'd' => s(Ins::POPFD),
                            'q' => s(Ins::POPFQ),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'u' => match rstr[1] {
                    's' => match rstr[2] {
                        'h' => match rstr[3] {
                            'f' => match rstr[4] {
                                'd' => s(Ins::PUSHFD),
                                'q' => s(Ins::PUSHFQ),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'r' => match rstr[1] {
                'c' => match rstr[2] {
                    'p' => match rstr[3] {
                        'p' => match rstr[4] {
                            's' => s(Ins::RCPPS),
                            _ => n(),
                        },
                        's' => match rstr[4] {
                            's' => s(Ins::RCPSS),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            _ => n(),
        },
        6 => match rstr[0] {
            'h' => match rstr[1] {
                's' => match rstr[2] {
                    'u' => match rstr[3] {
                        'b' => match rstr[4] {
                            'p' => match rstr[5] {
                                's' => s(Ins::HSUBPS),
                                'd' => s(Ins::HSUBPD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'a' => match rstr[2] {
                    'd' => match rstr[3] {
                        'd' => match rstr[4] {
                            'p' => match rstr[5] {
                                's' => s(Ins::HADDPS),
                                'd' => s(Ins::HADDPD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'l' => match rstr[1] {
                'f' => match rstr[2] {
                    'e' => match rstr[3] {
                        'n' => match rstr[4] {
                            'c' => match rstr[5] {
                                'e' => s(Ins::LFENCE),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'm' => match rstr[1] {
                'f' => match rstr[2] {
                    'e' => match rstr[3] {
                        'n' => match rstr[4] {
                            'c' => match rstr[5] {
                                'e' => s(Ins::MFENCE),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'o' => match rstr[2] {
                    'v' => match rstr[3] {
                        'n' => match rstr[4] {
                            't' => match rstr[5] {
                                'i' => s(Ins::MOVNTI),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'u' => match rstr[4] {
                            'p' => match rstr[5] {
                                'd' => s(Ins::MOVUPD),
                                's' => s(Ins::MOVUPS),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'h' => match rstr[4] {
                            'p' => match rstr[5] {
                                'd' => s(Ins::MOVHPD),
                                's' => s(Ins::MOVHPS),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'l' => match rstr[4] {
                            'p' => match rstr[5] {
                                'd' => s(Ins::MOVLPD),
                                's' => s(Ins::MOVLPS),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'a' => match rstr[4] {
                            'p' => match rstr[5] {
                                'd' => s(Ins::MOVAPD),
                                's' => s(Ins::MOVAPS),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'd' => match rstr[4] {
                            'q' => match rstr[5] {
                                'a' => s(Ins::MOVDQA),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'c' => match rstr[1] {
                'o' => match rstr[2] {
                    'm' => match rstr[3] {
                        'i' => match rstr[4] {
                            's' => match rstr[5] {
                                'd' => s(Ins::COMISD),
                                's' => s(Ins::COMISS),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'a' => match rstr[1] {
                'n' => match rstr[2] {
                    'd' => match rstr[3] {
                        'n' => match rstr[4] {
                            'p' => match rstr[5] {
                                's' => s(Ins::ANDNPS),
                                'd' => s(Ins::ANDNPD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            's' => match rstr[1] {
                'h' => match rstr[2] {
                    'u' => match rstr[3] {
                        'f' => match rstr[4] {
                            'p' => match rstr[5] {
                                's' => s(Ins::SHUFPS),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'q' => match rstr[2] {
                    'r' => match rstr[3] {
                        't' => match rstr[4] {
                            'p' => match rstr[5] {
                                's' => s(Ins::SQRTPS),
                                'd' => s(Ins::SQRTPD),
                                _ => n(),
                            },
                            's' => match rstr[5] {
                                's' => s(Ins::SQRTSS),
                                'd' => s(Ins::SQRTSD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'p' => match rstr[1] {
                's' => match rstr[2] {
                    'u' => match rstr[3] {
                        'b' => match rstr[4] {
                            's' => match rstr[5] {
                                'b' => s(Ins::PSUBSB),
                                'w' => s(Ins::PSUBSW),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'r' => match rstr[3] {
                        'l' => match rstr[4] {
                            'd' => match rstr[5] {
                                'q' => s(Ins::PSRLDQ),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'l' => match rstr[3] {
                        'l' => match rstr[4] {
                            'd' => match rstr[5] {
                                'q' => s(Ins::PSLLDQ),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'h' => match rstr[3] {
                        'u' => match rstr[4] {
                            'f' => match rstr[5] {
                                'd' => s(Ins::PSHUFD),
                                'b' => s(Ins::PSHUFB),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'i' => match rstr[3] {
                        'g' => match rstr[4] {
                            'n' => match rstr[5] {
                                'w' => s(Ins::PSIGNW),
                                'd' => s(Ins::PSIGND),
                                'b' => s(Ins::PSIGNB),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'u' => match rstr[2] {
                    's' => match rstr[3] {
                        'h' => match rstr[4] {
                            'f' => match rstr[5] {
                                'd' => s(Ins::PUSHFD),
                                'q' => s(Ins::PUSHFQ),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'a' => match rstr[2] {
                    'd' => match rstr[3] {
                        'd' => match rstr[4] {
                            's' => match rstr[5] {
                                'b' => s(Ins::PADDSB),
                                'w' => s(Ins::PADDSW),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'm' => match rstr[2] {
                    'u' => match rstr[3] {
                        'l' => match rstr[4] {
                            'l' => match rstr[5] {
                                'w' => s(Ins::PMULLW),
                                _ => n(),
                            },
                            'h' => match rstr[5] {
                                'w' => s(Ins::PMULHW),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'h' => match rstr[2] {
                    's' => match rstr[3] {
                        'u' => match rstr[4] {
                            'b' => match rstr[5] {
                                'w' => s(Ins::PHSUBW),
                                'd' => s(Ins::PHSUBD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'a' => match rstr[3] {
                        'd' => match rstr[4] {
                            'd' => match rstr[5] {
                                'w' => s(Ins::PHADDW),
                                'd' => s(Ins::PHADDD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            _ => n(),
        },
        7 => match rstr[0] {
            'p' => match rstr[1] {
                's' => match rstr[2] {
                    'u' => match rstr[3] {
                        'b' => match rstr[4] {
                            'u' => match rstr[5] {
                                's' => match rstr[6] {
                                    'b' => s(Ins::PSUBUSB),
                                    'w' => s(Ins::PSUBUSW),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'h' => match rstr[3] {
                        'u' => match rstr[4] {
                            'f' => match rstr[5] {
                                'l' => match rstr[6] {
                                    'w' => s(Ins::PSHUFLW),
                                    _ => n(),
                                },
                                'h' => match rstr[6] {
                                    'w' => s(Ins::PSHUFHW),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'm' => match rstr[2] {
                    'a' => match rstr[3] {
                        'd' => match rstr[4] {
                            'd' => match rstr[5] {
                                'w' => match rstr[6] {
                                    'd' => s(Ins::PMADDWD),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'u' => match rstr[3] {
                        'l' => match rstr[4] {
                            'u' => match rstr[5] {
                                'd' => match rstr[6] {
                                    'q' => s(Ins::PMULUDQ),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'h' => match rstr[2] {
                    'a' => match rstr[3] {
                        'd' => match rstr[4] {
                            'd' => match rstr[5] {
                                's' => match rstr[6] {
                                    'w' => s(Ins::PHADDSW),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    's' => match rstr[3] {
                        'u' => match rstr[4] {
                            'b' => match rstr[5] {
                                's' => match rstr[6] {
                                    'w' => s(Ins::PHSUBSW),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'a' => match rstr[2] {
                    'l' => match rstr[3] {
                        'i' => match rstr[4] {
                            'g' => match rstr[5] {
                                'n' => match rstr[6] {
                                    'r' => s(Ins::PALIGNR),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'c' => match rstr[2] {
                    'm' => match rstr[3] {
                        'p' => match rstr[4] {
                            'g' => match rstr[5] {
                                't' => match rstr[6] {
                                    'b' => s(Ins::PCMPGTB),
                                    'w' => s(Ins::PCMPGTW),
                                    'd' => s(Ins::PCMPGTD),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            'e' => match rstr[5] {
                                'q' => match rstr[6] {
                                    'b' => s(Ins::PCMPEQB),
                                    'w' => s(Ins::PCMPEQW),
                                    'd' => s(Ins::PCMPEQD),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'u' => match rstr[1] {
                'c' => match rstr[2] {
                    'o' => match rstr[3] {
                        'm' => match rstr[4] {
                            'i' => match rstr[5] {
                                's' => match rstr[6] {
                                    's' => s(Ins::UCOMISS),
                                    'd' => s(Ins::UCOMISD),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'r' => match rstr[1] {
                's' => match rstr[2] {
                    'q' => match rstr[3] {
                        'r' => match rstr[4] {
                            't' => match rstr[5] {
                                'p' => match rstr[6] {
                                    's' => s(Ins::RSQRTPS),
                                    _ => n(),
                                },
                                's' => match rstr[6] {
                                    's' => s(Ins::RSQRTSS),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'm' => match rstr[1] {
                'o' => match rstr[2] {
                    'n' => match rstr[3] {
                        'i' => match rstr[4] {
                            't' => match rstr[5] {
                                'o' => match rstr[6] {
                                    'r' => s(Ins::MONITOR),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'v' => match rstr[3] {
                        'n' => match rstr[4] {
                            't' => match rstr[5] {
                                'p' => match rstr[6] {
                                    'd' => s(Ins::MOVNTPD),
                                    _ => n(),
                                },
                                'd' => match rstr[6] {
                                    'q' => s(Ins::MOVNTDQ),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'd' => match rstr[4] {
                            'd' => match rstr[5] {
                                'u' => match rstr[6] {
                                    'p' => s(Ins::MOVDDUP),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            'q' => match rstr[5] {
                                '2' => match rstr[6] {
                                    'q' => s(Ins::MOVDQ2Q),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'q' => match rstr[4] {
                            '2' => match rstr[5] {
                                'd' => match rstr[6] {
                                    'q' => s(Ins::MOVQ2DQ),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'h' => match rstr[4] {
                            'l' => match rstr[5] {
                                'p' => match rstr[6] {
                                    's' => s(Ins::MOVHLPS),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'l' => match rstr[4] {
                            'h' => match rstr[5] {
                                'p' => match rstr[6] {
                                    's' => s(Ins::MOVLHPS),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            's' => match rstr[1] {
                'y' => match rstr[2] {
                    's' => match rstr[3] {
                        'c' => match rstr[4] {
                            'a' => match rstr[5] {
                                'l' => match rstr[6] {
                                    'l' => s(Ins::SYSCALL),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'c' => match rstr[1] {
                'l' => match rstr[2] {
                    'f' => match rstr[3] {
                        'l' => match rstr[4] {
                            'u' => match rstr[5] {
                                's' => match rstr[6] {
                                    'h' => s(Ins::CLFLUSH),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            _ => n(),
        },
        8 => match rstr[0] {
            'a' => match rstr[1] {
                'd' => match rstr[2] {
                    'd' => match rstr[3] {
                        's' => match rstr[4] {
                            'u' => match rstr[5] {
                                'b' => match rstr[6] {
                                    'p' => match rstr[7] {
                                        's' => s(Ins::ADDSUBPS),
                                        'd' => s(Ins::ADDSUBPD),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'm' => match rstr[1] {
                'o' => match rstr[2] {
                    'v' => match rstr[3] {
                        'm' => match rstr[4] {
                            's' => match rstr[5] {
                                'k' => match rstr[6] {
                                    'p' => match rstr[7] {
                                        'd' => s(Ins::MOVMSKPD),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        's' => match rstr[4] {
                            'l' => match rstr[5] {
                                'd' => match rstr[6] {
                                    'u' => match rstr[7] {
                                        'p' => s(Ins::MOVSLDUP),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            'h' => match rstr[5] {
                                'd' => match rstr[6] {
                                    'u' => match rstr[7] {
                                        'p' => s(Ins::MOVSHDUP),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'p' => match rstr[1] {
                'a' => match rstr[2] {
                    'c' => match rstr[3] {
                        'k' => match rstr[4] {
                            'u' => match rstr[5] {
                                's' => match rstr[6] {
                                    'w' => match rstr[7] {
                                        'b' => s(Ins::PACKUSWB),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            's' => match rstr[5] {
                                's' => match rstr[6] {
                                    'd' => match rstr[7] {
                                        'w' => s(Ins::PACKSSDW),
                                        _ => n(),
                                    },
                                    'w' => match rstr[7] {
                                        'b' => s(Ins::PACKSSWB),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'm' => match rstr[2] {
                    'u' => match rstr[3] {
                        'l' => match rstr[4] {
                            'h' => match rstr[5] {
                                'r' => match rstr[6] {
                                    's' => match rstr[7] {
                                        'w' => s(Ins::PMULHRSW),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'u' => match rstr[1] {
                'n' => match rstr[2] {
                    'p' => match rstr[3] {
                        'c' => match rstr[4] {
                            'k' => match rstr[5] {
                                'h' => match rstr[6] {
                                    'p' => match rstr[7] {
                                        's' => s(Ins::UNPCKHPS),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                'l' => match rstr[6] {
                                    'p' => match rstr[7] {
                                        's' => s(Ins::UNPCKLPS),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'c' => match rstr[1] {
                'v' => match rstr[2] {
                    't' => match rstr[3] {
                        'd' => match rstr[4] {
                            'q' => match rstr[5] {
                                '2' => match rstr[6] {
                                    'p' => match rstr[7] {
                                        'd' => s(Ins::CVTDQ2PD),
                                        's' => s(Ins::CVTDQ2PS),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        's' => match rstr[4] {
                            's' => match rstr[5] {
                                '2' => match rstr[6] {
                                    's' => match rstr[7] {
                                        'i' => s(Ins::CVTSS2SI),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            'd' => match rstr[5] {
                                '2' => match rstr[6] {
                                    's' => match rstr[7] {
                                        'i' => s(Ins::CVTSD2SI),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            'i' => match rstr[5] {
                                '2' => match rstr[6] {
                                    's' => match rstr[7] {
                                        's' => s(Ins::CVTSI2SS),
                                        'd' => s(Ins::CVTSI2SD),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'p' => match rstr[4] {
                            'd' => match rstr[5] {
                                '2' => match rstr[6] {
                                    'p' => match rstr[7] {
                                        'i' => s(Ins::CVTPD2PI),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            'i' => match rstr[5] {
                                '2' => match rstr[6] {
                                    'p' => match rstr[7] {
                                        'd' => s(Ins::CVTPI2PD),
                                        's' => s(Ins::CVTPI2PS),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            's' => match rstr[5] {
                                '2' => match rstr[6] {
                                    'd' => match rstr[7] {
                                        'q' => s(Ins::CVTPS2DQ),
                                        _ => n(),
                                    },
                                    'p' => match rstr[7] {
                                        'i' => s(Ins::CVTPS2PI),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            _ => n(),
        },
        9 => match rstr[0] {
            'p' => match rstr[1] {
                'm' => match rstr[2] {
                    'a' => match rstr[3] {
                        'd' => match rstr[4] {
                            'd' => match rstr[5] {
                                'u' => match rstr[6] {
                                    'b' => match rstr[7] {
                                        's' => match rstr[8] {
                                            'w' => s(Ins::PMADDUBSW),
                                            _ => n(),
                                        },
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'u' => match rstr[2] {
                    'n' => match rstr[3] {
                        'p' => match rstr[4] {
                            'c' => match rstr[5] {
                                'k' => match rstr[6] {
                                    'h' => match rstr[7] {
                                        'b' => match rstr[8] {
                                            'w' => s(Ins::PUNPCKHBW),
                                            _ => n(),
                                        },
                                        'w' => match rstr[8] {
                                            'd' => s(Ins::PUNPCKHWD),
                                            _ => n(),
                                        },
                                        'd' => match rstr[8] {
                                            'q' => s(Ins::PUNPCKHDQ),
                                            _ => n(),
                                        },
                                        _ => n(),
                                    },
                                    'l' => match rstr[7] {
                                        'b' => match rstr[8] {
                                            'w' => s(Ins::PUNPCKLBW),
                                            _ => n(),
                                        },
                                        'w' => match rstr[8] {
                                            'd' => s(Ins::PUNPCKLWD),
                                            _ => n(),
                                        },
                                        'd' => match rstr[8] {
                                            'q' => s(Ins::PUNPCKLDQ),
                                            _ => n(),
                                        },
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            'c' => match rstr[1] {
                'v' => match rstr[2] {
                    't' => match rstr[3] {
                        't' => match rstr[4] {
                            's' => match rstr[5] {
                                'd' => match rstr[6] {
                                    '2' => match rstr[7] {
                                        's' => match rstr[8] {
                                            'i' => s(Ins::CVTSD2SI),
                                            _ => n(),
                                        },
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            'p' => match rstr[5] {
                                'd' => match rstr[6] {
                                    '2' => match rstr[7] {
                                        'p' => match rstr[8] {
                                            'i' => s(Ins::CVTTPD2PI),
                                            _ => n(),
                                        },
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                's' => match rstr[6] {
                                    '2' => match rstr[7] {
                                        'p' => match rstr[8] {
                                            'i' => s(Ins::CVTTPS2PI),
                                            _ => n(),
                                        },
                                        'd' => match rstr[8] {
                                            'q' => s(Ins::CVTTPS2DQ),
                                            _ => n(),
                                        },
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            _ => n(),
        },
        10 => match rstr[0] {
            'm' => ins_ie(str, "maskmovdqu", Ins::MASKMOVDQU).ok(),
            'p' => match rstr[1] {
                'u' => match (rstr[6], rstr[7], rstr[8]) {
                    ('h', 'q', 'd') => ins_ie(str, "punpckhqdq", Ins::PUNPCKHQDQ).ok(),
                    ('l', 'q', 'd') => ins_ie(str, "punpcklqdq", Ins::PUNPCKLQDQ).ok(),
                    _ => n(),
                },
                _ => n(),
            },
            _ => n(),
        },
        _ => n(),
    }
}
