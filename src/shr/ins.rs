// pasm - src/shr/ins.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr;
use shr::size::Size;

#[cfg(not(feature = "refresh"))]
use shr::ins_switch;

#[cfg(not(feature = "refresh"))]
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
    //  - [x] avx-part2x
    //  - [x] FMA/AES
    //  - [x] conversions
    // hopefully i can finish before end of 31.05.2025 (spoiler: i did it :D)
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

    VPMULUDQ, VPMULHUW,
    
    VPMULHRSW,
    // part c extended - tests/*/avx-part2c-ext.asm
    PINSRW, PMAXSW, PMINSW, VPMAXUD, PMAXUD, PMULHUW,

    // this is FMA extension, but it uses VEX, so why not?
    // /tests/*/fma-part1.asm
    VFMADD132PD, VFMADD132PS, VFMADD132SS, VFMADD132SD,
    VFMADD213PD, VFMADD213PS, VFMADD213SS, VFMADD213SD,
    VFMADD231PD, VFMADD231PS, VFMADD231SS, VFMADD231SD,
    VFMSUB132PD, VFMSUB132PS, VFMSUB132SS, VFMSUB132SD,
    VFMSUB213PD, VFMSUB213PS, VFMSUB213SS, VFMSUB213SD,
    VFMSUB231PD, VFMSUB231PS, VFMSUB231SS, VFMSUB231SD,
    // /tests/*/fma-part2.asm
    VFNMADD132PD, VFNMADD132PS, VFNMADD132SS, VFNMADD132SD,
    VFNMADD213PD, VFNMADD213PS, VFNMADD213SS, VFNMADD213SD,
    VFNMADD231PD, VFNMADD231PS, VFNMADD231SS, VFNMADD231SD,
    VFNMSUB132PD, VFNMSUB132PS, VFNMSUB132SS, VFNMSUB132SD,
    VFNMSUB213PD, VFNMSUB213PS, VFNMSUB213SS, VFNMSUB213SD,
    VFNMSUB231PD, VFNMSUB231PS, VFNMSUB231SS, VFNMSUB231SD,
    // /tests/*/fma-part3.asm
    VFMADDSUB132PD, VFMADDSUB132PS,
    VFMADDSUB213PD, VFMADDSUB213PS,
    VFMADDSUB231PD, VFMADDSUB231PS,
    VFMSUBADD132PD, VFMSUBADD132PS,
    VFMSUBADD213PD, VFMSUBADD213PS,
    VFMSUBADD231PD, VFMSUBADD231PS,

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
    CVTPS2DQ, CVTPS2PD, CVTPS2PI, CVTPI2PD,
    CVTPI2PS, CVTPD2DQ, CVTPD2PI, CVTPD2PS,
    CVTDQ2PD, CVTDQ2PS,
    CVTSD2SI, CVTSD2SS, CVTSI2SD, 
    CVTSI2SS, CVTSS2SD, CVTSS2SI, 

    CVTTPD2DQ, CVTTPD2PI,
    CVTTPS2DQ, CVTTPS2PI, 
    CVTTSD2SI, CVTTSS2SI,

    // /tests/*/cvt-part2.asm
    VCVTPD2DQ, VCVTPD2PS,
    VCVTPS2DQ, VCVTPS2PD,
    VCVTSD2SI, VCVTSD2SS, VCVTSI2SD, VCVTSI2SS,
    VCVTSS2SD, VCVTSS2SI, VCVTDQ2PD, VCVTDQ2PS,

    VCVTTPD2DQ, VCVTTPS2DQ, VCVTTSD2SI, VCVTTSS2SI,

    // /tests/*/norm-part1X.asm
    // X = a
    BT,
    
    CLC, CMC, CWD, CDQ, CQO, DAA,
    DAS, CLD, CLI, AAA, AAD, AAM,
    AAS, ADC, BSF, BSR, BTC, BTR,
    BTS, CBW,

    // X = b
    ADCX, ADOX, ANDN, ARPL, BLSI, BLSR,
    BZHI, CWDE, CDQE, CLAC, CLTS, CLUI,
    CLWB,
    
    BEXTR, BSWAP,

    BLSMSK,
    
    // X = c
    CMPSTRB, CMPSTRD, CMPSTRQ, CMPSTRW, CMPXCHG,
    ENDBR32, ENDBR64,
    
    CLDEMOTE, CLRSSBSY,
    
    CMPXCHG8B,
    
    CMPXCHG16B,
    // /tests/*/norm-part2.asm
    ENTER, HLT, HRESET, 
    INSB, INSD, INSW, INT,
    INT3, INTO, INT1, INVD, INVLPG, INVPCID,
    IRET, IRETD, IRETQ, LAHF, LAR, LEAVE, LLDT, LMSW,
    LODSB, LODSW, LODSD, LODSQ,
    // /tests/*/norm-part3.asm
    LOOP, LOOPE, LOOPNE, LSL, LTR, LZCNT, MOVBE,
    MOVDIRI, MOVSTRB, MOVSTRW, MOVSTRD, MOVSTRQ,
    MOVZX, MULX, 
    OUTSB, OUTSD, OUTSW, 
    PEXT,
    PDEP, PREFETCHW, PREFETCH0, PREFETCH1, PREFETCH2, PREFETCHA,
    RCL, RCR, ROL, ROR,
    // /tests/*/norm-part4.asm
    RDMSR, RDPID, RDPKRU, RDPMC, RDRAND,
    RDSEED, RDSSPQ, RDSSPD, RDTSC, RDTSCP, RORX,
    RSM, RSTORSSP, SAHF, SARX, SHLX, SHRX, SBB,
    SCASB, SCASW, SCASD, SCASQ,
    SENDUIPI, SERIALIZE, SETSSBY,
    // /tests/*/setcc.asm
    SETA, SETAE, SETB, SETBE, SETC, SETE,
    SETG, SETGE, SETL, SETLE, SETNA, SETNAE,
    SETNB, SETNBE, SETNC, SETNE, SETNG, SETNL,
    SETNGE, SETNLE, SETNO, SETNP, SETNS, SETNZ, SETO,
    SETP, SETPE, SETPO, SETS, SETZ,
    // /tests/*/norm-part5.asm
    SFENCE, SHLD,
    SHRD, SMSW, STAC, STC, STD,
    STI, STOSB, STOSW, STOSD, STOSQ,
    STR, STUI, SYSENTER, SYSEXIT, SYSRET,
    TESTUI, TPAUSE, UD0, UD1, UD2, UIRET,
    UMONITOR, UMWAIT, VERR, VERW, WAIT, FWAIT,
    WBINVD, WRFSBASE, WRGSBASE, WRMSR,
    WRPKRU,
    // /tests/*/norm-part6.asm
    XABORT, XRELEASE, XACQUIRE, XADD,
    XBEGIN, XCHG, XEND, XGETBV, XLAT, XLATB, XLATB64, XRESLDTRK,
    XRSTOR, XRSTOR64, XRSTORS, XRSTORS64, XSAVE, XSAVE64, XSAVEC,
    XSAVEC64, XSAVEOPT, XSAVEOPT64, XSAVES, XSAVES64,
    XSETBV, XSUSLDTRK, XTEST,

    // /tests/*/sha.asm
    SHA1MSG1, SHA1MSG2,
    SHA1NEXTE, SHA1RNDS4,
    SHA256MSG1, SHA256MSG2,
    SHA256RNDS2,

    // not real x86-64 instructions
    
    // aliases to big endian variant
    BYTE, WORD, DWORD, QWORD,

    BYTEBE, WORDBE, DWORDBE, QWORDBE,
    BYTELE, WORDLE, DWORDLE, QWORDLE,
    
    EMPTY,

    ASCII, STRING,

    // fixed instructions
    IN, OUT, LGDT, LIDT,

    LOCK, REPNE, REPNZ, REPZ, REPE, REP,

    JCXZ, JECXZ, JRCXZ,

    // AVX-512 (with AVX-10)
    EADDPH, EADDSH, EALIGND, EALIGNQ, EBCSTNEBF162PS,
    EBCSTNESH2PS, EBLENDMPD, EBLENDMPS,
    EBROADCASTSD,
    EBROADCASTF32X2,
    EBROADCASTSS,
    EBROADCASTF32X4,
    EBROADCASTF64X2,
    EBROADCASTF32X8,
    EBROADCASTF64X4,
    ECMPPH,
    ECMPSH,
    ECOMISH,
    ECOMPRESSPD,
    ECOMPRESSPS,
    // this has no real purpose, but why not?
    __LAST
}

#[cfg(not(feature = "refresh"))]
impl FromStr for Mnemonic {
    type Err = ();
    fn from_str(str_ins: &str) -> Result<Self, <Self as FromStr>::Err> {
        if let Some(m) = ins_switch::mnem_fromstr(str_ins) {
            Ok(m)
        } else {
            Err(())
        }
    }
}

impl Mnemonic {
    pub fn allows_diff_size(&self, left: Option<Size>, right: Option<Size>) -> bool {
        if matches!(&self, Self::LAR) {
            let l = left.unwrap();
            let r = right.unwrap();
            l == Size::Word && r == Size::Dword
        } else if matches!(
            &self,
            Self::SHL
                | Self::SHR
                | Self::SAL
                | Self::SAR
                | Self::ROL
                | Self::ROR
                | Self::RCL
                | Self::RCR
        ) {
            let r = right.unwrap();
            r == Size::Byte
        } else {
            false
        }
    }
    pub fn allows_mem_mem(&self) -> bool {
        false
    }
    #[rustfmt::skip]
    pub fn defaults_to_64bit(&self) -> bool {
        matches!(
            self,
            Self::PUSH     | Self::POP     | Self::PADDB |
            Self::PADDW    | Self::PADDD   | Self::PADDQ |
            Self::PINSRD   | Self::CVTPD2PI| Self::CVTTPD2PI|
            Self::CVTTPS2PI| Self::CVTPI2PS| Self::CVTPS2PI|
            Self::CVTPS2DQ | Self::CVTSS2SD| Self::CVTPI2PD|
            Self::CVTPS2PD
        )
    }
}

#[cfg(feature = "iinfo")]
impl ToString for Mnemonic {
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}
