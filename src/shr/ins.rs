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
        let raw_ins = str_ins.as_bytes();
        match raw_ins.len() {
            1 => Err(()),
            2 => match raw_ins[1] as char {
                'e' => ins_ie(str_ins, "je", Self::JE),
                'z' => ins_ie(str_ins, "jz", Self::JZ),
                'l' => ins_ie(str_ins, "jl", Self::JL),
                'g' => ins_ie(str_ins, "jg", Self::JG),
                'r' => ins_ie(str_ins, "or", Self::OR),
                _ => Err(()),
            },
            3 => match raw_ins[1] as char {
                'o' => match raw_ins[0] as char {
                    'm' => ins_ie(str_ins, "mov", Self::MOV),
                    'n' => match raw_ins[2] as char {
                        't' => Ok(Self::NOT),
                        'p' => Ok(Self::NOP),
                        _ => Err(()),
                    },
                    'x' => ins_ie(str_ins, "xor", Self::XOR),
                    'p' => match raw_ins[2] as char {
                        'p' => Ok(Self::POP),
                        'r' => Ok(Self::POR),
                        _ => Err(()),
                    },
                    _ => Err(()),
                },
                'i' => ins_ie(str_ins, "div", Self::DIV),
                'd' => ins_ie(str_ins, "add", Self::ADD),
                'u' => match raw_ins[0] as char {
                    's' => ins_ie(str_ins, "sub", Self::SUB),
                    'm' => ins_ie(str_ins, "mul", Self::MUL),
                    _ => Err(()),
                },
                'e' => match raw_ins[0] as char {
                    'r' => ins_ie(str_ins, "ret", Self::RET),
                    'd' => ins_ie(str_ins, "dec", Self::DEC),
                    'l' => ins_ie(str_ins, "lea", Self::LEA),
                    'n' => ins_ie(str_ins, "neg", Self::NEG),
                    _ => Err(()),
                },
                'g' => ins_ie(str_ins, "jge", Self::JGE),
                'l' => ins_ie(str_ins, "jle", Self::JLE),
                'n' => match raw_ins[2] as char {
                    'c' => ins_ie(str_ins, "inc", Self::INC),
                    'd' => ins_ie(str_ins, "and", Self::AND),
                    'z' => ins_ie(str_ins, "jnz", Self::JNZ),
                    'e' => ins_ie(str_ins, "jne", Self::JNE),
                    _ => Err(()),
                },
                'h' => match raw_ins[2] as char {
                    'l' => ins_ie(str_ins, "shl", Self::SHL),
                    'r' => ins_ie(str_ins, "shr", Self::SHR),
                    _ => Err(()),
                },
                'a' => match raw_ins[2] as char {
                    'l' => ins_ie(str_ins, "sal", Self::SAL),
                    'r' => ins_ie(str_ins, "sar", Self::SAR),
                    _ => Err(()),
                },
                'm' => match raw_ins[0] as char {
                    'j' => ins_ie(str_ins, "jmp", Self::JMP),
                    'c' => ins_ie(str_ins, "cmp", Self::CMP),
                    _ => Err(()),
                },
                _ => Err(()),
            },
            4 => match raw_ins[1] as char {
                'd' => ins_ie(str_ins, "idiv", Self::IDIV),
                'm' => match raw_ins[0] as char {
                    'i' => ins_ie(str_ins, "imul", Self::IMUL),
                    'e' => ins_ie(str_ins, "emms", Self::EMMS),
                    _ => Err(()),
                },
                'u' => ins_ie(str_ins, "push", Self::PUSH),
                'a' => match raw_ins[0] as char {
                    'c' => ins_ie(str_ins, "call", Self::CALL),
                    'p' => ins_ie(str_ins, "pand", Self::PAND),
                    _ => Err(()),
                },
                'o' => match raw_ins[3] as char {
                    'f' => ins_ie(str_ins, "popf", Self::POPF),
                    'd' => ins_ie(str_ins, "movd", Self::MOVD),
                    'q' => ins_ie(str_ins, "movq", Self::MOVQ),
                    _ => Err(()),
                },
                'r' => ins_ie(str_ins, "orps", Self::ORPS),
                'e' => ins_ie(str_ins, "test", Self::TEST),
                'x' => ins_ie(str_ins, "pxor", Self::PXOR),
                _ => Err(()),
            },
            5 => match raw_ins[4] as char {
                's' => match raw_ins[2] as char {
                    'd' => match raw_ins[1] as char {
                        'd' => {
                            if raw_ins[3] as char == 'p' {
                                ins_ie(str_ins, "addps", Self::ADDPS)
                            } else {
                                ins_ie(str_ins, "addss", Self::ADDSS)
                            }
                        }
                        'n' => ins_ie(str_ins, "andps", Self::ANDPS),
                        _ => Err(()),
                    },
                    'b' => {
                        if raw_ins[3] as char == 'p' {
                            ins_ie(str_ins, "subps", Self::SUBPS)
                        } else {
                            ins_ie(str_ins, "subss", Self::SUBSS)
                        }
                    }
                    'l' => {
                        if raw_ins[3] as char == 'p' {
                            ins_ie(str_ins, "mulps", Self::MULPS)
                        } else {
                            ins_ie(str_ins, "mulss", Self::MULSS)
                        }
                    }
                    'v' => match (raw_ins[1] as char, raw_ins[3] as char) {
                        ('i', 'p') => ins_ie(str_ins, "divps", Self::DIVPS),
                        ('i', 's') => ins_ie(str_ins, "divss", Self::DIVSS),
                        ('o', 's') => ins_ie(str_ins, "movss", Self::MOVSS),
                        _ => Err(()),
                    },
                    'p' => match raw_ins[1] as char {
                        'c' => {
                            if raw_ins[3] as char == 'p' {
                                ins_ie(str_ins, "rcpps", Self::RCPPS)
                            } else {
                                ins_ie(str_ins, "rcpss", Self::RCPSS)
                            }
                        }
                        'm' => {
                            if raw_ins[3] as char == 'p' {
                                ins_ie(str_ins, "cmpps", Self::CMPPS)
                            } else {
                                ins_ie(str_ins, "cmpss", Self::CMPSS)
                            }
                        }
                        _ => Err(()),
                    },
                    'n' => {
                        if raw_ins[3] as char == 'p' {
                            ins_ie(str_ins, "minps", Self::MINPS)
                        } else {
                            ins_ie(str_ins, "minss", Self::MINSS)
                        }
                    }
                    'r' => ins_ie(str_ins, "xorps", Self::XORPS),
                    'x' => {
                        if raw_ins[3] as char == 'p' {
                            ins_ie(str_ins, "maxps", Self::MAXPS)
                        } else {
                            ins_ie(str_ins, "maxss", Self::MAXSS)
                        }
                    }
                    _ => Err(()),
                },
                'f' => ins_ie(str_ins, "pushf", Self::PUSHF),
                'n' => ins_ie(str_ins, "pandn", Self::PANDN),
                'd' => match raw_ins[3] as char {
                    'f' => ins_ie(str_ins, "popfd", Self::POPFD),
                    'l' => match raw_ins[2] as char {
                        'l' => ins_ie(str_ins, "pslld", Self::PSLLD),
                        'r' => ins_ie(str_ins, "psrld", Self::PSRLD),
                        _ => Err(()),
                    },
                    'a' => ins_ie(str_ins, "psrad", Self::PSRAD),
                    'd' => ins_ie(str_ins, "paddd", Self::PADDD),
                    'b' => ins_ie(str_ins, "psubd", Self::PSUBD),
                    'p' => match raw_ins[2] as char {
                        'd' => {
                            if raw_ins[1] as char == 'd' {
                                ins_ie(str_ins, "addpd", Self::ADDPD)
                            } else {
                                ins_ie(str_ins, "andpd", Self::ANDPD)
                            }
                        }
                        'b' => ins_ie(str_ins, "subpd", Self::SUBPD),
                        'l' => ins_ie(str_ins, "mulpd", Self::MULPD),
                        'v' => ins_ie(str_ins, "divpd", Self::DIVPD),
                        'x' => ins_ie(str_ins, "maxpd", Self::MAXPD),
                        'n' => ins_ie(str_ins, "minpd", Self::MINPD),
                        'p' => ins_ie(str_ins, "cmppd", Self::CMPPD),
                        'r' => ins_ie(str_ins, "xorpd", Self::XORPD),
                        _ => Err(()),
                    },
                    's' => match raw_ins[2] as char {
                        'd' => ins_ie(str_ins, "addsd", Self::ADDSD),
                        'b' => ins_ie(str_ins, "subsd", Self::SUBSD),
                        'l' => ins_ie(str_ins, "mulsd", Self::MULSD),
                        'v' => match raw_ins[0] as char {
                            'd' => ins_ie(str_ins, "divsd", Self::DIVSD),
                            'm' => ins_ie(str_ins, "movsd", Self::MOVSD),
                            _ => Err(()),
                        },
                        'x' => ins_ie(str_ins, "maxsd", Self::MAXSD),
                        'n' => ins_ie(str_ins, "minsd", Self::MINSD),
                        'p' => ins_ie(str_ins, "cmpsd", Self::CMPSD),
                        _ => Err(()),
                    },
                    _ => Err(()),
                },
                'b' => match raw_ins[3] as char {
                    'b' => ins_ie(str_ins, "psubb", Self::PSUBB),
                    'd' => ins_ie(str_ins, "paddb", Self::PADDB),
                    _ => Err(()),
                },
                'w' => match raw_ins[3] as char {
                    'b' => ins_ie(str_ins, "psubw", Self::PSUBW),
                    'd' => ins_ie(str_ins, "paddw", Self::PADDW),
                    'a' => ins_ie(str_ins, "psraw", Self::PSRAW),
                    'l' => match raw_ins[2] as char {
                        'l' => ins_ie(str_ins, "psllw", Self::PSLLW),
                        'r' => match raw_ins[3] as char {
                            'l' => ins_ie(str_ins, "psrlw", Self::PSRLW),
                            _ => Err(()),
                        },
                        _ => Err(()),
                    },
                    _ => Err(()),
                },
                'q' => match raw_ins[3] as char {
                    'f' => ins_ie(str_ins, "popfq", Self::POPFQ),
                    'd' => ins_ie(str_ins, "paddq", Self::PADDQ),
                    'l' => match raw_ins[2] as char {
                        'l' => ins_ie(str_ins, "psllq", Self::PSLLQ),
                        'r' => ins_ie(str_ins, "psrlq", Self::PSRLQ),
                        _ => Err(()),
                    },
                    _ => Err(()),
                },
                'i' => ins_ie(str_ins, "cpuid", Self::CPUID),
                _ => Err(()),
            },
            6 => match raw_ins[5] as char {
                'd' => match raw_ins[4] as char {
                    'f' => ins_ie(str_ins, "pushfd", Self::PUSHFD),
                    'p' => match raw_ins[3] as char {
                        'n' => ins_ie(str_ins, "andnpd", Self::ANDNPD),
                        'a' => ins_ie(str_ins, "movapd", Self::MOVAPD),
                        'u' => ins_ie(str_ins, "movupd", Self::MOVUPD),
                        'l' => ins_ie(str_ins, "movlpd", Self::MOVLPD),
                        'h' => ins_ie(str_ins, "movhpd", Self::MOVHPD),
                        't' => ins_ie(str_ins, "sqrtpd", Self::SQRTPD),
                        _ => Err(()),
                    },
                    's' => match raw_ins[3] as char {
                        'i' => ins_ie(str_ins, "comisd", Self::COMISD),
                        't' => ins_ie(str_ins, "sqrtsd", Self::SQRTSD),
                        _ => Err(()),
                    },
                    _ => Err(()),
                },
                'b' => match raw_ins[3] as char {
                    'b' => ins_ie(str_ins, "psubsb", Self::PSUBSB),
                    'd' => ins_ie(str_ins, "paddsb", Self::PADDSB),
                    _ => Err(()),
                },
                'w' => match (raw_ins[3] as char, raw_ins[4] as char) {
                    ('l', 'l') => ins_ie(str_ins, "pmullw", Self::PMULLW),
                    ('l', 'h') => ins_ie(str_ins, "pmulhw", Self::PMULHW),
                    ('b', 's') => ins_ie(str_ins, "psubsw", Self::PSUBSW),
                    ('d', 's') => ins_ie(str_ins, "paddsw", Self::PADDSW),
                    _ => Err(()),
                },
                'q' => ins_ie(str_ins, "pushfq", Self::PUSHFQ),
                's' => match raw_ins[1] as char {
                    'q' => {
                        if raw_ins[4] as char == 'p' {
                            ins_ie(str_ins, "sqrtps", Self::SQRTPS)
                        } else {
                            ins_ie(str_ins, "sqrtss", Self::SQRTSS)
                        }
                    }
                    'c' => ins_ie(str_ins, "comiss", Self::COMISS),
                    'n' => ins_ie(str_ins, "andnps", Self::ANDPS),
                    'o' => match raw_ins[3] as char {
                        'a' => ins_ie(str_ins, "movaps", Self::MOVAPS),
                        'l' => ins_ie(str_ins, "movlps", Self::MOVLPS),
                        'h' => ins_ie(str_ins, "movhps", Self::MOVHPS),
                        'u' => ins_ie(str_ins, "movups", Self::MOVUPS),
                        _ => Err(()),
                    },
                    'h' => ins_ie(str_ins, "shufps", Self::SHUFPS),
                    _ => Err(()),
                },
                'a' => ins_ie(str_ins, "movdqa", Self::MOVDQA),
                _ => Err(()),
            },
            7 => match raw_ins[0] as char {
                's' => ins_ie(str_ins, "syscall", Self::SYSCALL),
                'u' => match raw_ins[6] as char {
                    'd' => ins_ie(str_ins, "ucomisd", Self::UCOMISD),
                    's' => ins_ie(str_ins, "ucomiss", Self::UCOMISS),
                    _ => Err(()),
                },
                'p' => match raw_ins[4] as char {
                    'd' => ins_ie(str_ins, "pmaddwd", Self::PMADDWD),
                    'g' => match raw_ins[6] as char {
                        'b' => ins_ie(str_ins, "pcmpgtb", Self::PCMPGTB),
                        'w' => ins_ie(str_ins, "pcmpgtw", Self::PCMPGTW),
                        'd' => ins_ie(str_ins, "pcmpgtd", Self::PCMPGTD),
                        _ => Err(()),
                    },
                    'e' => match raw_ins[6] as char {
                        'b' => ins_ie(str_ins, "pcmpeqb", Self::PCMPEQB),
                        'w' => ins_ie(str_ins, "pcmpeqw", Self::PCMPEQW),
                        'd' => ins_ie(str_ins, "pcmpeqd", Self::PCMPEQD),
                        _ => Err(()),
                    },
                    'u' => match raw_ins[1] as char {
                        'a' => match raw_ins[6] as char {
                            'b' => ins_ie(str_ins, "paddusb", Self::PADDUSB),
                            'w' => ins_ie(str_ins, "paddusw", Self::PADDUSW),
                            _ => Err(()),
                        },
                        's' => match raw_ins[6] as char {
                            'b' => ins_ie(str_ins, "psubusb", Self::PSUBUSB),
                            'w' => ins_ie(str_ins, "psubusw", Self::PSUBUSW),
                            _ => Err(()),
                        },
                        _ => Err(()),
                    },
                    _ => Err(()),
                },
                'r' => {
                    if raw_ins[5] as char == 'p' {
                        ins_ie(str_ins, "rsqrtps", Self::RSQRTPS)
                    } else {
                        ins_ie(str_ins, "rsqrtss", Self::RSQRTSS)
                    }
                }
                'm' => match raw_ins[3] as char {
                    'l' => ins_ie(str_ins, "movlhps", Self::MOVLHPS),
                    'h' => ins_ie(str_ins, "movhlps", Self::MOVHLPS),
                    'q' => match raw_ins[5] as char {
                        'd' => ins_ie(str_ins, "movq2dq", Self::MOVQ2DQ),
                        _ => Err(()),
                    },
                    'd' => ins_ie(str_ins, "movdq2q", Self::MOVDQ2Q),
                    _ => Err(()),
                },
                _ => Err(()),
            },
            8 => match raw_ins[0] as char {
                'c' => match (raw_ins[3] as char, raw_ins[4] as char, raw_ins[7] as char) {
                    ('s', 'i', 's') => ins_ie(str_ins, "cvtsi2ss", Self::CVTSI2SS),
                    ('p', 'i', 's') => ins_ie(str_ins, "cvtpi2ps", Self::CVTPI2PS),
                    ('s', 's', 'i') => ins_ie(str_ins, "cvtss2si", Self::CVTSS2SI),
                    ('p', 's', 'i') => ins_ie(str_ins, "cvtps2pi", Self::CVTPS2PI),
                    _ => Err(()),
                },
                'u' => match raw_ins[5] as char {
                    'h' => ins_ie(str_ins, "unpckhps", Self::UNPCKHPS),
                    'l' => ins_ie(str_ins, "unpcklps", Self::UNPCKLPS),
                    _ => Err(()),
                },
                'p' => match raw_ins[1] as char {
                    'a' => match raw_ins[4] as char {
                        'u' => ins_ie(str_ins, "packuswb", Self::PACKUSWB),
                        's' => match raw_ins[7] as char {
                            'w' => ins_ie(str_ins, "packssdw", Self::PACKSSDW),
                            'b' => ins_ie(str_ins, "packsswb", Self::PACKSSWB),
                            _ => Err(()),
                        },
                        _ => Err(()),
                    },
                    _ => Err(()),
                },
                'm' => ins_ie(str_ins, "movmskpd", Self::MOVMSKPD),
                _ => Err(()),
            },
            9 => match raw_ins[0] as char {
                'c' => match raw_ins[8] as char {
                    'i' => ins_ie(str_ins, "cvttps2pi", Self::CVTTPS2PI),
                    _ => Err(()),
                },
                'p' => match raw_ins[1] as char {
                    'u' => match (raw_ins[6] as char, raw_ins[7] as char, raw_ins[8] as char) {
                        ('l', 'b', 'w') => ins_ie(str_ins, "punpcklbw", Self::PUNPCKLBW),
                        ('l', 'w', 'd') => ins_ie(str_ins, "punpcklwd", Self::PUNPCKLWD),
                        ('l', 'd', 'q') => ins_ie(str_ins, "punpckldq", Self::PUNPCKLDQ),
                        ('h', 'b', 'w') => ins_ie(str_ins, "punpckhbw", Self::PUNPCKHBW),
                        ('h', 'w', 'd') => ins_ie(str_ins, "punpckhwd", Self::PUNPCKHWD),
                        ('h', 'd', 'q') => ins_ie(str_ins, "punpckhdq", Self::PUNPCKHDQ),
                        _ => Err(()),
                    },
                    _ => Err(()),
                },
                _ => Err(()),
            },
            _ => Err(()),
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
