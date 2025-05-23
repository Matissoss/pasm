// rasmx86_64 - src/shr/ins.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

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

    SYSCALL, RET,
    NOP,

    POP   , POPF  , POPFD,
    POPFQ , PUSH  , PUSHF,
    PUSHFD, PUSHFQ,

    CPUID,
    
    // Jcc
    JA, JC, JE, JZ, JL,
    JG, JO, JP, JS, JB,
    
    JAE, JBE, JNZ, JNE,
    JNO, JNP, JNS, JPE,
    JPO, JLE, JGE, JNA,
    JNB, JNC, JNL, JNG,
    
    JNAE, JNBE, JNGE, JNLE,

    // CMOVcc
    CMOVA, CMOVB, CMOVC, CMOVE, CMOVG,
    CMOVL, CMOVO, CMOVP, CMOVS, CMOVZ,

    CMOVAE, CMOVBE, CMOVGE, CMOVLE, CMOVNA,
    CMOVNB, CMOVNC, CMOVNE, CMOVNG, CMOVNL,
    CMOVNO, CMOVNP, CMOVNS, CMOVNZ, CMOVPE, CMOVPO,
    
    CMOVNBE, CMOVNAE, CMOVNGE,CMOVNLE,

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

    PSUBQ  , PSHUFD    , PSLLDQ,
    PSRLDQ , PMULUDQ   , PSHUFLW,
    PSHUFHW, PUNPCKHQDQ, PUNPCKLQDQ,

    MASKMOVDQU,

    MFENCE , LFENCE , CLFLUSH, PAUSE,
    MOVNTPD, MOVNTDQ, MOVNTI,

    // SSE3 extension
    ADDSUBPS, ADDSUBPD,

    HADDPS, HSUBPS,
    HADDPD, HSUBPD,
    MOVSLDUP, MOVSHDUP,
    MOVDDUP, LDDQU,
    MONITOR, MWAIT,

    // SSSE3 extension

    PABSW, PABSD, PABSB,
    PSIGNW, PSIGND, PSIGNB, PHSUBW,
    PHSUBD, PHADDW, PHADDD, PSHUFB, PHSUBSW,
    PHADDSW, PALIGNR, PMULHRSW, PMADDUBSW,

    // MMX extension
    MOVD  , MOVQ,
    PADDB , PADDW  , PADDD  , PADDQ  ,
    PADDSB, PADDSW , PADDUSB, PADDUSW,
    PSUBB , PSUBW  , PSUBD  , PSUBSB ,
    PSUBSW, PSUBUSB, PSUBUSW, PANDN  ,
    PMULHW, PMULLW,
    PMADDWD,
    PCMPEQB, PCMPEQW, PCMPEQD,
    PCMPGTB, PCMPGTW, PCMPGTD,
    PACKUSWB, PACKSSWB, PACKSSDW,
    PUNPCKLBW, PUNPCKLWD, PUNPCKLDQ,
    PUNPCKHBW, PUNPCKHWD, PUNPCKHDQ,
    POR, PAND, PXOR,
    PSLLW, PSLLD, PSLLQ,
    PSRLW, PSRLD, PSRLQ,
    PSRAW, PSRAD,

    EMMS,

    // SSE4_1 and SSE4_2
    DPPS, DPPD,
    
    PTEST, CRC32,
    
    PEXTRB, PEXTRW, PEXTRD, PEXTRQ,
    PINSRB, PINSRD, PINSRQ, PMAXSB,
    PMAXSD, PMAXUW, PMINSB, PMINSD,
    PMINUW,
    PMULDQ, PMULLD, POPCNT,
    
    BLENDPS, BLENDPD, PBLENDW, PCMPEQQ,
    ROUNDPD, ROUNDPS, ROUNDSD, ROUNDSS,
    MPSADBW, PCMPGTQ,
    
    BLENDVPS, BLENDVPD, PBLENDVB, INSERTPS,
    PACKUSDW, PCMPESTRI,
    MOVNTDQA,

    EXTRACTPS, PCMPESTRM, PCMPISTRI, PCMPISTRM,
    
    PHMINPOSUW,

    // AVX/AVX2
    // ---
    // AVX support roadmap:
    //  - [x] SSE/MMX derived
    //  - [ ] avx-part2x
    //  - [ ] FMA/AES
    //  - [ ] conversions
    // hopefully i can finish before end of 31.05.2025
    // ---
    // derived from SSE
    VMOVAPS , VMOVUPS, 
    VADDPS  , VADDSS,
    VSUBPS  , VSUBSS,
    VMULPS  , VMULSS,
    VDIVPS  , VDIVSS,
    VRCPPS  , VRCPSS,
    VSQRTPS , VSQRTSS,
    VRSQRTPS, VRSQRTSS,
    VMINPS  , VMINSS,
    VMAXPS  , VMAXSS,

    VORPS   , VANDPS,
    VANDNPS , VXORPS,
    VCMPPS  , VCMPSS,

    VCOMISS , VUCOMISS,

    VSHUFPS, VUNPCKLPS, VUNPCKHPS,

    VMOVSS, VMOVLPS, VMOVHPS,
    
    VMOVLHPS,VMOVHLPS,

    // derived from SSE2
    VADDPD  , VADDSD,
    VSUBPD  , VSUBSD,
    VMULPD  , VMULSD,
    VDIVPD  , VDIVSD,
    VSQRTPD , VSQRTSD,
    VMINPD  , VMINSD,
    VMAXPD  , VMAXSD,
    VORPD   , VANDPD,
    VANDNPD , VXORPD,

    VCMPPD  , VCMPSD,
    VCOMISD , VUCOMISD,

    VMOVAPD , VMOVUPD,
    VMOVHPD , VMOVLPD,
    VMOVSD  , VMOVMSKPD,

    VMOVDQA,
    
    // derived from SSE3 extension
    VADDSUBPS, VADDSUBPD,

    VHADDPS, VHSUBPS,
    VHADDPD, VHSUBPD,
    VMOVSLDUP, VMOVSHDUP,
    VMOVDDUP, VLDDQU,
    
    // derived from SSE4_1 and SSE4_2
    VDPPS, VDPPD,
    
    VPTEST,
    
    VPEXTRB, VPEXTRW, VPEXTRD, VPEXTRQ,
    VPINSRB, VPINSRD, VPINSRQ, VPMAXSB,
    VPMAXSD, VPMAXUW, VPMINSB, VPMINSD,
    VPMINUW, VPMULDQ, VPMULLD, VPMAXUB,
    VPMINUB,
    
    VBLENDPS, VBLENDPD, VPBLENDW, VPCMPEQQ,
    VROUNDPD, VROUNDPS, VROUNDSD, VROUNDSS,
    VMPSADBW, VPCMPGTQ,
    
    VBLENDVPS, VBLENDVPD, VPBLENDVB, VINSERTPS,
    VPACKUSDW, VPCMPESTRI,
    VMOVNTDQA,

    VEXTRACTPS, VPCMPESTRM, VPCMPISTRI, VPCMPISTRM,
    
    VPHMINPOSUW,

    // derived from MMX
    
    // part 1
    VPOR,
    VMOVD, VMOVQ, VPAND, VPXOR,
    VPADDB, VPADDW, VPADDD, VPADDQ,
    VPSUBB, VPSUBW, VPSUBD, VPSUBQ,
    VPANDN,
    VPSLLW, VPSLLD, VPSLLQ, VPSRLW,
    VPSRLD, VPSRLQ, VPSRAW, VPSRAD,
    
    VPSUBSB, VPMULHW, VPMULLW,
    VPADDSB, VPADDSW, VPSUBSW,

    // part 2
    VPSUBUSB, VPADDUSB, VPADDUSW, VPSUBUSW,
    VPMADDWD, VPCMPEQB, VPCMPEQW, VPCMPEQD,
    VPCMPGTB, VPCMPGTW, VPCMPGTD, 

    VPACKUSWB, VPACKSSWB, VPACKSSDW,
    
    VPUNPCKLBW, VPUNPCKLWD, VPUNPCKLDQ,
    VPUNPCKHBW, VPUNPCKHWD, VPUNPCKHDQ,

    // AVX/AVX2 (and even SSE ;) ) part 2
    // part a - tests/*/avx-part2a.asm
    PAVGB, PAVGW,
    VPAVGB, VPAVGW,

    VPHADDW, VPHADDD, VPHSUBW, VPHSUBD,
    
    VZEROALL, VPALIGNR,

    VZEROUPPER,
    VINSERTF128, VEXTRACTF128, VBROADCASTSS, VBROADCASTSD,

    VBROADCASTF128,

    // part b - tests/*/avx-part2b.asm
    STMXCSR, LDMXCSR,
    
    VLDMXCSR, VSTMXCSR,
    
    VMOVMSKPS,
    
    VPERMILPD, VPERMILPS, PCLMULQDQ, 
    VPCLMULQDQ,

    VPERM2F128, VPERM2I128,

    // part c - tests/*/avx-part2c.asm
    VPINSRW, VPMAXSW, VPMINSW,
    VPSRLDQ, VPSIGND, VPSIGNB, VPSIGNW,

    VPMAXUSB, VPMAXUSW, VPSQLLDQ,
    VPMAXUSD, VPMINUSB, VPHSUFLW,
    VPMINUSW, VPMINUSD, VPMULUDQ, VPMULHUW,
    
    VPHSHUFHW, VPMULHRSW,

    // this is FMA extension, but it uses VEX, so why not?
    // /tests/*/fma.asm
    VFMADDPD, VFMADDPS, VFMADDSS, VFMADDSD,
    VFMSUBPD, VFMSUBPS, VFMSUBSS, VFMSUBSD,
    VFNMADDPD, VFNMADDPS, VFNMADDSS, VFNMADDSD,
    VFNMSUBPD, VFNMSUBPS, VFNMSUBSS, VFNMSUBSD,
    VFMADDSUBPD, VFMADDSUBPS,
    VFMSUBADDPD, VFMSUBADDPS,

    // same but AES
    // /tests/*/aes.asm
    AESDEC, AESENC, AESIMC,
    
    VAESDEC, VAESENC, VAESIMC,
    
    AESDECLAST, AESENCLAST,
    
    VAESDECLAST, VAESENCLAST,
    
    AESKEYGENASSIST,
    VAESKEYGENASSIST, // 16 chars :o

    // i hate coding conversions, but i guess someone has to do them and actually test them :)
    // so here are conversions from MMX/SSE/AVX
    // /tests/*/cvt-part1.asm
    CVTDQ2PD, CVTDQ2PS, CVTPD2DQ, CVTPD2PI, CVTPD2PS, CVTPI2PD, CVTPI2PS,
    CVTPS2DQ, CVTPS2PD, CVTPS2PI, CVTSD2SI, CVTSD2SS, CVTSI2SD, CVTSI2SS,
    CVTSS2SD, CVTSS2SI, 

    CVTTPD2DQ, CVTTPD2PI, CVTTPS2DQ, CVTTPS2PI, CVTTSD2SI, CVTTSS2SI,

    // /tests/*/cvt-part2.asm
    VCVTDQ2PD, VCVTDQ2PS, VCVTPD2DQ, VCVTPD2PI, VCVTPD2PS, VCVTPI2PD, VCVTPI2PS,
    VCVTPS2DQ, VCVTPS2PD, VCVTPS2PI, VCVTSD2SI, VCVTSD2SS, VCVTSI2SD, VCVTSI2SS,
    VCVTSS2SD, VCVTSS2SI,

    VCVTTPD2DQ, VCVTTPD2PI, VCVTTPS2DQ, VCVTTPS2PI, VCVTTSD2SI, VCVTTSS2SI,

    // this has no real purpose, but why not?
    __LAST
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
    pub fn is_avx(&self) -> bool {
        format!("{:?}", self).starts_with('V')
    }
    #[rustfmt::skip]
    pub fn defaults_to_64bit(&self) -> bool {
        matches!(
            self,
            Self::PUSH | Self::POP   | Self::PADDB |
            Self::PADDW| Self::PADDD | Self::PADDQ |
            Self::PINSRD
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
                'a' => s(Ins::JA),
                'b' => s(Ins::JB),
                'c' => s(Ins::JC),
                'e' => s(Ins::JE),
                'z' => s(Ins::JZ),
                'l' => s(Ins::JL),
                'o' => s(Ins::JO),
                'p' => s(Ins::JP),
                's' => s(Ins::JS),
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
                'a' => match rstr[2] {
                    'e' => s(Ins::JAE),
                    _ => n(),
                },
                'b' => match rstr[2] {
                    'e' => s(Ins::JBE),
                    _ => n(),
                },
                'p' => match rstr[2] {
                    'o' => s(Ins::JPO),
                    'e' => s(Ins::JPE),
                    _ => n(),
                },
                'n' => match rstr[2] {
                    'a' => s(Ins::JNA),
                    'b' => s(Ins::JNB),
                    'c' => s(Ins::JNC),
                    'g' => s(Ins::JNG),
                    'l' => s(Ins::JNL),
                    'o' => s(Ins::JNO),
                    'p' => s(Ins::JNP),
                    's' => s(Ins::JNS),
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
            'v' => ins_ie(&rstr, 1, &cc::<4>("vpor"), Ins::VPOR),
            'j' => match rstr[1] {
                'n' => match rstr[2] {
                    'a' => match rstr[3] {
                        'e' => s(Ins::JNAE),
                        _ => n(),
                    },
                    'b' => match rstr[3] {
                        'e' => s(Ins::JNBE),
                        _ => n(),
                    },
                    'g' => match rstr[3] {
                        'e' => s(Ins::JNGE),
                        _ => n(),
                    },
                    'l' => match rstr[3] {
                        'e' => s(Ins::JNLE),
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
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
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            _ => n(),
        },
        5 => match rstr[0] {
            'v' => match rstr[1] {
                'm' => match rstr[2] {
                    'o' => match rstr[3] {
                        'v' => match rstr[4] {
                            'd' => s(Ins::VMOVD),
                            'q' => s(Ins::VMOVQ),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'p' => match rstr[2] {
                    'x' => match rstr[3] {
                        'o' => match rstr[4] {
                            'r' => s(Ins::VPXOR),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'a' => match rstr[3] {
                        'n' => match rstr[4] {
                            'd' => s(Ins::VPAND),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'd' => match rstr[2] {
                    'p' => match rstr[3] {
                        'p' => match rstr[4] {
                            's' => s(Ins::VDPPS),
                            'd' => s(Ins::VDPPD),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'o' => match rstr[2] {
                    'r' => match rstr[3] {
                        'p' => match rstr[4] {
                            's' => s(Ins::VORPS),
                            'd' => s(Ins::VORPD),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
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
                'r' => match rstr[2] {
                    'c' => match rstr[3] {
                        '3' => match rstr[4] {
                            '2' => s(Ins::CRC32),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'm' => match rstr[2] {
                    'o' => match rstr[3] {
                        'v' => match rstr[4] {
                            'a' => s(Ins::CMOVA),
                            'b' => s(Ins::CMOVB),
                            'c' => s(Ins::CMOVC),
                            'e' => s(Ins::CMOVE),
                            'g' => s(Ins::CMOVG),
                            'l' => s(Ins::CMOVL),
                            'o' => s(Ins::CMOVO),
                            'p' => s(Ins::CMOVP),
                            's' => s(Ins::CMOVS),
                            'z' => s(Ins::CMOVZ),
                            _ => n(),
                        },
                        _ => n(),
                    },
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
                't' => match rstr[2] {
                    'e' => match rstr[3] {
                        's' => match rstr[4] {
                            't' => s(Ins::PTEST),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'a' => match rstr[2] {
                    'v' => match rstr[3] {
                        'g' => match rstr[4] {
                            'b' => s(Ins::PAVGB),
                            'w' => s(Ins::PAVGW),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'n' => match rstr[3] {
                        'd' => match rstr[4] {
                            'n' => s(Ins::PANDN),
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
                'm' => match rstr[2] {
                    'o' => match rstr[3] {
                        'v' => match rstr[4] {
                            'a' => match rstr[5] {
                                'e' => s(Ins::CMOVAE),
                                _ => n(),
                            },
                            'b' => match rstr[5] {
                                'e' => s(Ins::CMOVBE),
                                _ => n(),
                            },
                            'l' => match rstr[5] {
                                'e' => s(Ins::CMOVLE),
                                _ => n(),
                            },
                            'p' => match rstr[5] {
                                'e' => s(Ins::CMOVPE),
                                'o' => s(Ins::CMOVPO),
                                _ => n(),
                            },
                            'g' => match rstr[5] {
                                'e' => s(Ins::CMOVGE),
                                _ => n(),
                            },
                            'n' => match rstr[5] {
                                'a' => s(Ins::CMOVNA),
                                'b' => s(Ins::CMOVNB),
                                'c' => s(Ins::CMOVNC),
                                'e' => s(Ins::CMOVNE),
                                'g' => s(Ins::CMOVNG),
                                'l' => s(Ins::CMOVNL),
                                'o' => s(Ins::CMOVNO),
                                'p' => s(Ins::CMOVNP),
                                's' => s(Ins::CMOVNS),
                                'z' => s(Ins::CMOVNZ),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
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
                'i' => match rstr[2] {
                    'n' => match rstr[3] {
                        's' => match rstr[4] {
                            'r' => match rstr[5] {
                                'b' => s(Ins::PINSRB),
                                'd' => s(Ins::PINSRD),
                                'q' => s(Ins::PINSRQ),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
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
                    'i' => match rstr[3] {
                        'n' => match rstr[4] {
                            's' => match rstr[5] {
                                'd' => s(Ins::PMINSD),
                                'b' => s(Ins::PMINSB),
                                _ => n(),
                            },
                            'u' => ins_ie(&rstr, 5, &cc::<6>("pminuw"), Ins::PMINUW),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'a' => match rstr[3] {
                        'x' => match rstr[4] {
                            's' => match rstr[5] {
                                'd' => s(Ins::PMAXSD),
                                'b' => s(Ins::PMAXSB),
                                _ => n(),
                            },
                            'u' => ins_ie(&rstr, 5, &cc::<6>("pmaxuw"), Ins::PMAXUW),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'u' => match rstr[3] {
                        'l' => match rstr[4] {
                            'd' => ins_ie(&rstr, 5, &cc::<6>("pmuldq"), Ins::PMULDQ),
                            'l' => match rstr[5] {
                                'w' => s(Ins::PMULLW),
                                'd' => s(Ins::PMULLD),
                                _ => n(),
                            },
                            'h' => ins_ie(&rstr, 5, &cc::<6>("pmulhw"), Ins::PMULHW),
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
                'e' => match rstr[2] {
                    'x' => match rstr[3] {
                        't' => match rstr[4] {
                            'r' => match rstr[5] {
                                'b' => s(Ins::PEXTRB),
                                'w' => s(Ins::PEXTRW),
                                'd' => s(Ins::PEXTRD),
                                'q' => s(Ins::PEXTRQ),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'o' => ins_ie(&rstr, 2, &cc::<6>("popcnt"), Ins::POPCNT),
                _ => n(),
            },
            'v' => match rstr[1] {
                'l' => ins_ie(&rstr, 2, &cc::<6>("vlddqu"), Ins::VLDDQU),
                'a' => match rstr[2] {
                    'n' => match rstr[3] {
                        'd' => match rstr[4] {
                            'p' => match rstr[5] {
                                's' => s(Ins::VANDPS),
                                'd' => s(Ins::VANDPD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'd' => match rstr[3] {
                        'd' => match rstr[4] {
                            's' => match rstr[5] {
                                's' => s(Ins::VADDSS),
                                'd' => s(Ins::VADDSD),
                                _ => n(),
                            },
                            'p' => match rstr[5] {
                                's' => s(Ins::VADDPS),
                                'd' => s(Ins::VADDPD),
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
                            's' => match rstr[5] {
                                's' => s(Ins::VCMPSS),
                                'd' => s(Ins::VCMPSD),
                                _ => n(),
                            },
                            'p' => match rstr[5] {
                                's' => s(Ins::VCMPPS),
                                'd' => s(Ins::VCMPPD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                's' => match rstr[2] {
                    'u' => match rstr[3] {
                        'b' => match rstr[4] {
                            's' => match rstr[5] {
                                's' => s(Ins::VSUBSS),
                                'd' => s(Ins::VSUBSD),
                                _ => n(),
                            },
                            'p' => match rstr[5] {
                                's' => s(Ins::VSUBPS),
                                'd' => s(Ins::VSUBPD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'm' => match rstr[2] {
                    'o' => match rstr[3] {
                        'v' => match rstr[4] {
                            's' => match rstr[5] {
                                's' => s(Ins::VMOVSS),
                                'd' => s(Ins::VMOVSD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'a' => match rstr[3] {
                        'x' => match rstr[4] {
                            's' => match rstr[5] {
                                's' => s(Ins::VMAXSS),
                                'd' => s(Ins::VMAXSD),
                                _ => n(),
                            },
                            'p' => match rstr[5] {
                                's' => s(Ins::VMAXPS),
                                'd' => s(Ins::VMAXPD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'i' => match rstr[3] {
                        'n' => match rstr[4] {
                            's' => match rstr[5] {
                                's' => s(Ins::VMINSS),
                                'd' => s(Ins::VMINSD),
                                _ => n(),
                            },
                            'p' => match rstr[5] {
                                's' => s(Ins::VMINPS),
                                'd' => s(Ins::VMINPD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'u' => match rstr[3] {
                        'l' => match rstr[4] {
                            's' => match rstr[5] {
                                's' => s(Ins::VMULSS),
                                'd' => s(Ins::VMULSD),
                                _ => n(),
                            },
                            'p' => match rstr[5] {
                                's' => s(Ins::VMULPS),
                                'd' => s(Ins::VMULPD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'd' => match rstr[2] {
                    'i' => match rstr[3] {
                        'v' => match rstr[4] {
                            's' => match rstr[5] {
                                's' => s(Ins::VDIVSS),
                                'd' => s(Ins::VDIVSD),
                                _ => n(),
                            },
                            'p' => match rstr[5] {
                                's' => s(Ins::VDIVPS),
                                'd' => s(Ins::VDIVPD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'x' => match rstr[2] {
                    'o' => match rstr[3] {
                        'r' => match rstr[4] {
                            'p' => match rstr[5] {
                                's' => s(Ins::VXORPS),
                                'd' => s(Ins::VXORPD),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'r' => match rstr[2] {
                    'c' => match rstr[3] {
                        'p' => match rstr[4] {
                            's' => match rstr[5] {
                                's' => s(Ins::VRCPSS),
                                _ => n(),
                            },
                            'p' => match rstr[5] {
                                's' => s(Ins::VRCPPS),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'p' => match rstr[2] {
                    't' => ins_ie(&rstr, 3, &cc::<6>("vptest"), Ins::VPTEST),
                    's' => match rstr[3] {
                        'r' => match rstr[4] {
                            'a' => match rstr[5] {
                                'w' => s(Ins::VPSRAW),
                                'd' => s(Ins::VPSRAD),
                                _ => n(),
                            },
                            'l' => match rstr[5] {
                                'w' => s(Ins::VPSRLW),
                                'd' => s(Ins::VPSRLD),
                                'q' => s(Ins::VPSRLQ),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'l' => match rstr[4] {
                            'l' => match rstr[5] {
                                'w' => s(Ins::VPSLLW),
                                'd' => s(Ins::VPSLLD),
                                'q' => s(Ins::VPSLLQ),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'u' => match rstr[4] {
                            'b' => match rstr[5] {
                                'b' => s(Ins::VPSUBB),
                                'w' => s(Ins::VPSUBW),
                                'd' => s(Ins::VPSUBD),
                                'q' => s(Ins::VPSUBQ),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'a' => match rstr[3] {
                        'v' => match rstr[4] {
                            'g' => match rstr[5] {
                                'b' => s(Ins::VPAVGB),
                                'w' => s(Ins::VPAVGW),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'n' => match rstr[4] {
                            'd' => match rstr[5] {
                                'n' => s(Ins::VPANDN),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'd' => match rstr[4] {
                            'd' => match rstr[5] {
                                'b' => s(Ins::VPADDB),
                                'w' => s(Ins::VPADDW),
                                'd' => s(Ins::VPADDD),
                                'q' => s(Ins::VPADDQ),
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
                    'a' => ins_ie(&rstr, 3, &cc::<7>("pmaddwd"), Ins::PMADDWD),
                    'u' => ins_ie(&rstr, 3, &cc::<7>("pmuludq"), Ins::PMULUDQ),
                    _ => n(),
                },
                'h' => match rstr[2] {
                    'a' => ins_ie(&rstr, 3, &cc::<7>("phaddsw"), Ins::PHADDSW),
                    's' => ins_ie(&rstr, 3, &cc::<7>("phsubsw"), Ins::PHSUBSW),
                    _ => n(),
                },
                'a' => ins_ie(&rstr, 2, &cc::<7>("palignr"), Ins::PALIGNR),
                'c' => match rstr[2] {
                    'm' => match rstr[3] {
                        'p' => match rstr[4] {
                            'g' => match rstr[5] {
                                't' => match rstr[6] {
                                    'b' => s(Ins::PCMPGTB),
                                    'w' => s(Ins::PCMPGTW),
                                    'd' => s(Ins::PCMPGTD),
                                    'q' => s(Ins::PCMPGTQ),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            'e' => match rstr[5] {
                                'q' => match rstr[6] {
                                    'b' => s(Ins::PCMPEQB),
                                    'w' => s(Ins::PCMPEQW),
                                    'd' => s(Ins::PCMPEQD),
                                    'q' => s(Ins::PCMPEQQ),
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
                'b' => ins_ie(&rstr, 2, &cc::<7>("pblendw"), Ins::PBLENDW),
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
                'o' => match rstr[2] {
                    'u' => match rstr[3] {
                        'n' => match rstr[4] {
                            'd' => match rstr[5] {
                                's' => match rstr[6] {
                                    's' => s(Ins::ROUNDSS),
                                    'd' => s(Ins::ROUNDSD),
                                    _ => n(),
                                },
                                'p' => match rstr[6] {
                                    's' => s(Ins::ROUNDPS),
                                    'd' => s(Ins::ROUNDPD),
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
                'p' => ins_ie(&rstr, 2, &cc::<7>("mpsadbw"), Ins::MPSADBW),
                'o' => match rstr[2] {
                    'n' => ins_ie(&rstr, 3, &cc::<7>("monitor"), Ins::MONITOR),
                    'v' => match rstr[3] {
                        'n' => match rstr[4] {
                            't' => match rstr[5] {
                                'p' => ins_ie(&rstr, 6, &cc::<7>("movntpd"), Ins::MOVNTPD),
                                'd' => ins_ie(&rstr, 6, &cc::<7>("movntdq"), Ins::MOVNTDQ),
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'd' => match rstr[4] {
                            'd' => ins_ie(&rstr, 5, &cc::<7>("movddup"), Ins::MOVDDUP),
                            'q' => ins_ie(&rstr, 5, &cc::<7>("movdq2q"), Ins::MOVDQ2Q),
                            _ => n(),
                        },
                        'q' => ins_ie(&rstr, 4, &cc::<7>("movq2dq"), Ins::MOVQ2DQ),
                        'h' => ins_ie(&rstr, 4, &cc::<7>("movhlps"), Ins::MOVHLPS),
                        'l' => ins_ie(&rstr, 4, &cc::<7>("movlhps"), Ins::MOVLHPS),
                        _ => n(),
                    },
                    _ => n(),
                },
                _ => n(),
            },
            's' => match rstr[1] {
                'y' => ins_ie(&rstr, 2, &cc::<7>("syscall"), Ins::SYSCALL),
                't' => ins_ie(&rstr, 2, &cc::<7>("stmxcsr"), Ins::STMXCSR),
                _ => n(),
            },
            'l' => ins_ie(&rstr, 2, &cc::<7>("ldmxcsr"), Ins::LDMXCSR),
            'c' => match rstr[1] {
                'm' => match rstr[2] {
                    'o' => match rstr[3] {
                        'v' => match rstr[4] {
                            'n' => match rstr[5] {
                                'a' => match rstr[6] {
                                    'e' => s(Ins::CMOVNAE),
                                    _ => n(),
                                },
                                'l' => match rstr[6] {
                                    'e' => s(Ins::CMOVNLE),
                                    _ => n(),
                                },
                                'g' => match rstr[6] {
                                    'e' => s(Ins::CMOVNGE),
                                    _ => n(),
                                },
                                'b' => match rstr[6] {
                                    'e' => s(Ins::CMOVNBE),
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
                'l' => ins_ie(&rstr, 2, &cc::<7>("clflush"), Ins::CLFLUSH),
                _ => n(),
            },
            'b' => match rstr[1] {
                'l' => match rstr[2] {
                    'e' => match rstr[3] {
                        'n' => match rstr[4] {
                            'd' => match rstr[5] {
                                'p' => match rstr[6] {
                                    's' => s(Ins::BLENDPS),
                                    'd' => s(Ins::BLENDPD),
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
            'v' => match rstr[1] {
                'h' => match rstr[2] {
                    's' => match rstr[3] {
                        'u' => match rstr[4] {
                            'b' => match rstr[5] {
                                'p' => match rstr[6] {
                                    's' => s(Ins::VHSUBPS),
                                    'd' => s(Ins::VHSUBPD),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'a' => match rstr[3] {
                        'd' => match rstr[4] {
                            'd' => match rstr[5] {
                                'p' => match rstr[6] {
                                    's' => s(Ins::VHADDPS),
                                    'd' => s(Ins::VHADDPD),
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
                    'o' => match rstr[3] {
                        'm' => match rstr[4] {
                            'i' => match rstr[5] {
                                's' => match rstr[6] {
                                    's' => s(Ins::VCOMISS),
                                    'd' => s(Ins::VCOMISD),
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
                's' => match rstr[2] {
                    'h' => match rstr[3] {
                        'u' => match rstr[4] {
                            'f' => match rstr[5] {
                                'p' => match rstr[6] {
                                    's' => s(Ins::VSHUFPS),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'q' => match rstr[3] {
                        'r' => match rstr[4] {
                            't' => match rstr[5] {
                                'p' => match rstr[6] {
                                    's' => s(Ins::VSQRTPS),
                                    'd' => s(Ins::VSQRTPD),
                                    _ => n(),
                                },
                                's' => match rstr[6] {
                                    's' => s(Ins::VSQRTSS),
                                    'd' => s(Ins::VSQRTSD),
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
                    'n' => match rstr[3] {
                        'd' => match rstr[4] {
                            'n' => match rstr[5] {
                                'p' => match rstr[6] {
                                    's' => s(Ins::VANDNPS),
                                    'd' => s(Ins::VANDNPD),
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
                    'o' => match rstr[3] {
                        'v' => match rstr[4] {
                            'd' => match rstr[5] {
                                'q' => match rstr[6] {
                                    'a' => s(Ins::VMOVDQA),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            'h' => match rstr[5] {
                                'p' => match rstr[6] {
                                    's' => s(Ins::VMOVHPS),
                                    'd' => s(Ins::VMOVHPD),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            'l' => match rstr[5] {
                                'p' => match rstr[6] {
                                    's' => s(Ins::VMOVLPS),
                                    'd' => s(Ins::VMOVLPD),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            'u' => match rstr[5] {
                                'p' => match rstr[6] {
                                    's' => s(Ins::VMOVUPS),
                                    'd' => s(Ins::VMOVUPD),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            'a' => match rstr[5] {
                                'p' => match rstr[6] {
                                    's' => s(Ins::VMOVAPS),
                                    'd' => s(Ins::VMOVAPD),
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
                'p' => match rstr[2] {
                    'h' => match rstr[3] {
                        'a' => match rstr[4] {
                            'd' => match rstr[5] {
                                'd' => match rstr[6] {
                                    'w' => s(Ins::VPHADDW),
                                    'd' => s(Ins::VPHADDD),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        's' => match rstr[4] {
                            'u' => match rstr[5] {
                                'b' => match rstr[6] {
                                    'w' => s(Ins::VPHSUBW),
                                    'd' => s(Ins::VPHSUBD),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'a' => match rstr[3] {
                        'd' => match rstr[4] {
                            'd' => match rstr[5] {
                                's' => match rstr[6] {
                                    'b' => s(Ins::VPADDSB),
                                    'w' => s(Ins::VPADDSW),
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
                                    'b' => s(Ins::VPSUBSB),
                                    'w' => s(Ins::VPSUBSW),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'm' => match rstr[3] {
                        'i' => match rstr[4] {
                            'n' => match rstr[5] {
                                's' => match rstr[6] {
                                    'b' => s(Ins::VPMINSB),
                                    'd' => s(Ins::VPMINSD),
                                    _ => n(),
                                },
                                'u' => match rstr[6] {
                                    'w' => s(Ins::VPMINUW),
                                    'b' => s(Ins::VPMINUB),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'u' => match rstr[4] {
                            'l' => match rstr[5] {
                                'h' => match rstr[6] {
                                    'w' => s(Ins::VPMULHW),
                                    _ => n(),
                                },
                                'd' => match rstr[6] {
                                    'q' => s(Ins::VPMULDQ),
                                    _ => n(),
                                },
                                'l' => match rstr[6] {
                                    'd' => s(Ins::VPMULLD),
                                    'w' => s(Ins::VPMULLW),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        'a' => match rstr[4] {
                            'x' => match rstr[5] {
                                's' => match rstr[6] {
                                    'b' => s(Ins::VPMAXSB),
                                    'd' => s(Ins::VPMAXSD),
                                    _ => n(),
                                },
                                'u' => match rstr[6] {
                                    'w' => s(Ins::VPMAXUW),
                                    'b' => s(Ins::VPMAXUB),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'i' => match rstr[3] {
                        'n' => match rstr[4] {
                            's' => match rstr[5] {
                                'r' => match rstr[6] {
                                    'b' => s(Ins::VPINSRB),
                                    'd' => s(Ins::VPINSRD),
                                    'q' => s(Ins::VPINSRQ),
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'e' => match rstr[3] {
                        'x' => match rstr[4] {
                            't' => match rstr[5] {
                                'r' => match rstr[6] {
                                    'b' => s(Ins::VPEXTRB),
                                    'w' => s(Ins::VPEXTRW),
                                    'd' => s(Ins::VPEXTRD),
                                    'q' => s(Ins::VPEXTRQ),
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
                        'n' => match rstr[4] {
                            't' => match rstr[5] {
                                'd' => match rstr[6] {
                                    'q' => match rstr[7] {
                                        'a' => s(Ins::MOVNTDQA),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
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
                                    'd' => match rstr[7] {
                                        'w' => s(Ins::PACKUSDW),
                                        _ => n(),
                                    },
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
                'm' => ins_ie(&rstr, 2, &cc::<8>("pmulhrsw"), Ins::PMULHRSW),
                'b' => ins_ie(&rstr, 2, &cc::<8>("pblendvb"), Ins::PBLENDVB),
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
                            's' => ins_ie(&rstr, 5, &cc::<8>("cvtss2si"), Ins::CVTSS2SI),
                            'd' => ins_ie(&rstr, 5, &cc::<8>("cvtsd2si"), Ins::CVTSD2SI),
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
                            'd' => ins_ie(&rstr, 5, &cc::<8>("cvtpd2pi"), Ins::CVTPD2PI),
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
                                    'd' => ins_ie(&rstr, 7, &cc::<8>("cvtps2dq"), Ins::CVTPS2DQ),
                                    'p' => ins_ie(&rstr, 7, &cc::<8>("cvtps2pi"), Ins::CVTPS2PI),
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
            'i' => ins_ie(&rstr, 1, &cc::<8>("insertps"), Ins::INSERTPS),
            'b' => match rstr[1] {
                'l' => match rstr[2] {
                    'e' => match rstr[3] {
                        'n' => match rstr[4] {
                            'd' => match rstr[5] {
                                'v' => match rstr[6] {
                                    'p' => match rstr[7] {
                                        's' => s(Ins::BLENDVPS),
                                        'd' => s(Ins::BLENDVPD),
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
            'v' => match rstr[1] {
                's' => ins_ie(&rstr, 2, &cc::<8>("vstmxcsr"), Ins::VSTMXCSR),
                'l' => ins_ie(&rstr, 2, &cc::<8>("vldmxcsr"), Ins::VLDMXCSR),
                'z' => ins_ie(&rstr, 2, &cc::<8>("vzeroall"), Ins::VZEROALL),
                'm' => match rstr[2] {
                    'p' => ins_ie(&rstr, 3, &cc::<8>("vmpsadbw"), Ins::VMPSADBW),
                    'o' => match rstr[3] {
                        'v' => match rstr[4] {
                            'd' => ins_ie(&rstr, 5, &cc::<8>("vmovddup"), Ins::VMOVDDUP),
                            'h' => ins_ie(&rstr, 5, &cc::<8>("vmovhlps"), Ins::VMOVHLPS),
                            'l' => ins_ie(&rstr, 5, &cc::<8>("vmovlhps"), Ins::VMOVLHPS),
                            _ => n(),
                        },
                        _ => n(),
                    },
                    _ => n(),
                },
                'p' => match rstr[2] {
                    'm' => ins_ie(&rstr, 3, &cc::<8>("vpmaddwd"), Ins::VPMADDWD),
                    'a' => match rstr[3] {
                        'l' => ins_ie(&rstr, 4, &cc::<8>("vpalignr"), Ins::VPALIGNR),
                        'd' => match rstr[4] {
                            'd' => match rstr[5] {
                                'u' => match rstr[6] {
                                    's' => match rstr[7] {
                                        'b' => s(Ins::VPADDUSB),
                                        'w' => s(Ins::VPADDUSW),
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
                    's' => match rstr[3] {
                        'u' => match rstr[4] {
                            'b' => match rstr[5] {
                                'u' => match rstr[6] {
                                    's' => match rstr[7] {
                                        'b' => s(Ins::VPSUBUSB),
                                        'w' => s(Ins::VPSUBUSW),
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
                    'c' => match rstr[3] {
                        'm' => match rstr[4] {
                            'p' => match rstr[5] {
                                'g' => match rstr[6] {
                                    't' => match rstr[7] {
                                        'q' => s(Ins::VPCMPGTQ),
                                        'd' => s(Ins::VPCMPGTD),
                                        'w' => s(Ins::VPCMPGTW),
                                        'b' => s(Ins::VPCMPGTB),
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                'e' => match rstr[6] {
                                    'q' => match rstr[7] {
                                        'q' => s(Ins::VPCMPEQQ),
                                        'd' => s(Ins::VPCMPEQD),
                                        'w' => s(Ins::VPCMPEQW),
                                        'b' => s(Ins::VPCMPEQB),
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
                    'b' => ins_ie(&rstr, 3, &cc::<8>("vpblendw"), Ins::VPBLENDW),
                    _ => n(),
                },
                'b' => match rstr[2] {
                    'l' => match rstr[3] {
                        'e' => match rstr[4] {
                            'n' => match rstr[5] {
                                'd' => match rstr[6] {
                                    'p' => match rstr[7] {
                                        's' => s(Ins::VBLENDPS),
                                        'd' => s(Ins::VBLENDPD),
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
                    'c' => match rstr[3] {
                        'o' => match rstr[4] {
                            'm' => match rstr[5] {
                                'i' => match rstr[6] {
                                    's' => match rstr[7] {
                                        's' => s(Ins::VUCOMISS),
                                        'd' => s(Ins::VUCOMISD),
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
                'r' => match rstr[2] {
                    'o' => match rstr[3] {
                        'u' => match rstr[4] {
                            'n' => match rstr[5] {
                                'd' => match rstr[6] {
                                    's' => match rstr[7] {
                                        's' => s(Ins::VROUNDSS),
                                        'd' => s(Ins::VROUNDSD),
                                        _ => n(),
                                    },
                                    'p' => match rstr[7] {
                                        's' => s(Ins::VROUNDPS),
                                        'd' => s(Ins::VROUNDPD),
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
                    's' => match rstr[3] {
                        'q' => match rstr[4] {
                            'r' => match rstr[5] {
                                't' => match rstr[6] {
                                    's' => match rstr[7] {
                                        's' => s(Ins::VRSQRTSS),
                                        _ => n(),
                                    },
                                    'p' => match rstr[7] {
                                        's' => s(Ins::VRSQRTPS),
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
                'c' => match rstr[2] {
                    'l' => ins_ie(&rstr, 3, &cc::<9>("pclmulqdq"), Ins::PCLMULQDQ),
                    'm' => match rstr[3] {
                        'p' => match rstr[4] {
                            'i' => match rstr[5] {
                                's' => match rstr[6] {
                                    't' => match rstr[7] {
                                        'r' => match rstr[8] {
                                            'i' => s(Ins::PCMPISTRI),
                                            'm' => s(Ins::PCMPISTRM),
                                            _ => n(),
                                        },
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            'e' => match rstr[5] {
                                's' => match rstr[6] {
                                    't' => match rstr[7] {
                                        'r' => match rstr[8] {
                                            'i' => s(Ins::PCMPESTRI),
                                            'm' => s(Ins::PCMPESTRM),
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
                'm' => ins_ie(&rstr, 2, &cc::<9>("pmaddubsw"), Ins::PMADDUBSW),
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
                            'p' => match rstr[5] {
                                'd' => ins_ie(&rstr, 6, &cc::<9>("cvttpd2pi"), Ins::CVTTPD2PI),
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
            'e' => ins_ie(&rstr, 1, &cc::<9>("extractps"), Ins::EXTRACTPS),
            'v' => match rstr[1] {
                'i' => ins_ie(&rstr, 2, &cc::<9>("vinsertps"), Ins::VINSERTPS),
                'p' => match rstr[2] {
                    'b' => ins_ie(&rstr, 3, &cc::<9>("vpblendvb"), Ins::VPBLENDVB),
                    'a' => match rstr[3] {
                        'c' => match rstr[4] {
                            'k' => match rstr[5] {
                                'u' => match rstr[6] {
                                    's' => match rstr[7] {
                                        'w' => match rstr[8] {
                                            'b' => s(Ins::VPACKUSWB),
                                            _ => n(),
                                        },
                                        'd' => match rstr[8] {
                                            'w' => s(Ins::VPACKUSDW),
                                            _ => n(),
                                        },
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                's' => match rstr[6] {
                                    's' => match rstr[7] {
                                        'w' => match rstr[8] {
                                            'b' => s(Ins::VPACKSSWB),
                                            _ => n(),
                                        },
                                        'd' => match rstr[8] {
                                            'w' => s(Ins::VPACKSSDW),
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
                    'e' => match rstr[8] {
                        'd' => ins_ie(&rstr, 3, &cc::<9>("vpermilpd"), Ins::VPERMILPD),
                        's' => ins_ie(&rstr, 3, &cc::<9>("vpermilps"), Ins::VPERMILPS),
                        _ => n(),
                    },
                    _ => n(),
                },
                'b' => match rstr[2] {
                    'l' => match rstr[3] {
                        'e' => match rstr[4] {
                            'n' => match rstr[5] {
                                'd' => match rstr[6] {
                                    'v' => match rstr[7] {
                                        'p' => match rstr[8] {
                                            's' => s(Ins::VBLENDVPS),
                                            'd' => s(Ins::VBLENDVPD),
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
                'm' => match rstr[2] {
                    'o' => match rstr[3] {
                        'v' => match rstr[4] {
                            'n' => ins_ie(&rstr, 5, &cc::<9>("vmovntdqa"), Ins::VMOVNTDQA),
                            'm' => match rstr[8] {
                                's' => ins_ie(&rstr, 5, &cc::<9>("vmovmskps"), Ins::VMOVMSKPS),
                                'd' => ins_ie(&rstr, 5, &cc::<9>("vmovmskpd"), Ins::VMOVMSKPD),
                                _ => n(),
                            },
                            's' => match rstr[5] {
                                'l' => ins_ie(&rstr, 6, &cc::<9>("vmovsldup"), Ins::VMOVSLDUP),
                                'h' => ins_ie(&rstr, 6, &cc::<9>("vmovshdup"), Ins::VMOVSHDUP),
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
                                'u' => match rstr[6] {
                                    'b' => match rstr[7] {
                                        'p' => match rstr[8] {
                                            's' => s(Ins::VADDSUBPS),
                                            'd' => s(Ins::VADDSUBPD),
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
                                        'p' => match rstr[8] {
                                            's' => s(Ins::VUNPCKHPS),
                                            _ => n(),
                                        },
                                        _ => n(),
                                    },
                                    'l' => match rstr[7] {
                                        'p' => match rstr[8] {
                                            's' => s(Ins::VUNPCKLPS),
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
            'm' => ins_ie(&rstr, 1, &cc::<10>("maskmovdqu"), Ins::MASKMOVDQU),
            'p' => match rstr[1] {
                'h' => ins_ie(&rstr, 2, &cc::<10>("phminposuw"), Ins::PHMINPOSUW),
                'u' => match (rstr[6], rstr[7], rstr[8]) {
                    ('h', 'q', 'd') => ins_ie(&rstr, 1, &cc::<10>("punpckhqdq"), Ins::PUNPCKHQDQ),
                    ('l', 'q', 'd') => ins_ie(&rstr, 1, &cc::<10>("punpcklqdq"), Ins::PUNPCKLQDQ),
                    _ => n(),
                },
                _ => n(),
            },
            'v' => match rstr[1] {
                'z' => ins_ie(&rstr, 2, &cc::<10>("vzeroupper"), Ins::VZEROUPPER),
                'p' => match rstr[2] {
                    'e' => match rstr[3] {
                        'r' => match rstr[4] {
                            'm' => match rstr[5] {
                                '2' => match rstr[6] {
                                    'f' => {
                                        ins_ie(&rstr, 7, &cc::<10>("vperm2f128"), Ins::VPERM2F128)
                                    }
                                    'i' => {
                                        ins_ie(&rstr, 7, &cc::<10>("vperm2i128"), Ins::VPERM2I128)
                                    }
                                    _ => n(),
                                },
                                _ => n(),
                            },
                            _ => n(),
                        },
                        _ => n(),
                    },
                    'u' => match rstr[3] {
                        'n' => match rstr[4] {
                            'p' => match rstr[5] {
                                'c' => match rstr[6] {
                                    'k' => match rstr[7] {
                                        'h' => match rstr[8] {
                                            'b' => match rstr[9] {
                                                'w' => s(Ins::VPUNPCKHBW),
                                                _ => n(),
                                            },
                                            'd' => match rstr[9] {
                                                'q' => s(Ins::VPUNPCKHDQ),
                                                _ => n(),
                                            },
                                            'w' => match rstr[9] {
                                                'd' => s(Ins::VPUNPCKHWD),
                                                _ => n(),
                                            },
                                            _ => n(),
                                        },
                                        'l' => match rstr[8] {
                                            'd' => match rstr[9] {
                                                'q' => s(Ins::VPUNPCKLDQ),
                                                _ => n(),
                                            },
                                            'w' => match rstr[9] {
                                                'd' => s(Ins::VPUNPCKLWD),
                                                _ => n(),
                                            },
                                            'b' => match rstr[9] {
                                                'w' => s(Ins::VPUNPCKLBW),
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
                    'c' => match rstr[3] {
                        'l' => ins_ie(&rstr, 4, &cc::<10>("vpclmulqdq"), Ins::VPCLMULQDQ),
                        'm' => match rstr[4] {
                            'p' => match rstr[5] {
                                'i' => match rstr[6] {
                                    's' => match rstr[7] {
                                        't' => match rstr[8] {
                                            'r' => match rstr[9] {
                                                'm' => s(Ins::VPCMPISTRM),
                                                'i' => s(Ins::VPCMPISTRI),
                                                _ => n(),
                                            },
                                            _ => n(),
                                        },
                                        _ => n(),
                                    },
                                    _ => n(),
                                },
                                'e' => match rstr[6] {
                                    's' => match rstr[7] {
                                        't' => match rstr[8] {
                                            'r' => match rstr[9] {
                                                'm' => s(Ins::VPCMPESTRM),
                                                'i' => s(Ins::VPCMPESTRI),
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
                'e' => ins_ie(&rstr, 2, &cc::<10>("vextractps"), Ins::VEXTRACTPS),
                _ => n(),
            },
            _ => n(),
        },
        11 => match rstr[0] {
            'v' => match rstr[1] {
                'p' => ins_ie(&rstr, 2, &cc::<11>("vphminposuw"), Ins::VPHMINPOSUW),
                'i' => ins_ie(&rstr, 2, &cc::<11>("vinsertf128"), Ins::VINSERTF128),
                'e' => ins_ie(&rstr, 2, &cc::<11>("vextractf128"), Ins::VINSERTF128),
                _ => n(),
            },
            _ => n(),
        },
        12 => match rstr[0] {
            'v' => match rstr[1] {
                'e' => ins_ie(&rstr, 2, &cc::<12>("vextractf128"), Ins::VEXTRACTF128),
                'b' => match rstr[11] {
                    'd' => ins_ie(&rstr, 2, &cc::<12>("vbroadcastsd"), Ins::VBROADCASTSD),
                    's' => ins_ie(&rstr, 2, &cc::<12>("vbroadcastss"), Ins::VBROADCASTSS),
                    _ => n(),
                },
                _ => n(),
            },
            _ => n(),
        },
        14 => ins_ie(&rstr, 1, &cc::<14>("vbroadcastf128"), Ins::VBROADCASTF128),
        _ => n(),
    }
}

#[inline(always)]
const fn cc_for_const(rep_n: usize, byte_arr: &'static [u8], str: &mut [char]) {
    if rep_n == 0 {
        return;
    }
    str[rep_n] = byte_arr[rep_n] as char;
    cc_for_const(rep_n - 1, byte_arr, str)
}

#[inline(always)]
const fn cc<const N: usize>(str: &'static str) -> [char; N] {
    assert!(N == str.len());
    let mut arr = [' '; N];
    cc_for_const(N - 1, str.as_bytes(), &mut arr);
    arr
}

#[inline(always)]
fn ins_ie(chars: &[char], start: usize, target: &[char], res: Mnemonic) -> Option<Mnemonic> {
    if chars[start..] == target[start..] {
        Some(res)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ins_test() {
        println!(
            "This version of RASM supports {} x86-64 instructions!",
            Mnemonic::__LAST as u64 - 1
        );
        assert!(mnem_fromstr("vphminposuw") == Some(Mnemonic::VPHMINPOSUW));
    }
}
