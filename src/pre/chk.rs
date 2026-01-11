// pasm - src/pre/chk.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::pre::chkn;

use crate::core::apx::*;
use crate::shr::{
    atype::*,
    error::Error,
    instruction::{Instruction, Operand},
    mnemonic::Mnemonic,
    reg::Purpose as RPurpose,
    size::Size,
};

pub fn check_ins32bit(ins: &Instruction) -> Result<(), Error> {
    use Mnemonic::*;
    if ins.needs_rex() {
        let er = Error::new(
            "you tried to use instruction that requires REX prefix, but bits != 64",
            10,
        );
        return Err(er);
    } else if ins.needs_evex() {
        let er = Error::new(
            "you tried to use instruction that requires EVEX prefix, but bits != 64",
            10,
        );
        return Err(er);
    } else if ins.needs_apx_extension() {
        return Err(Error::new(
            "you tried to use instruction that requires APX extension, but bits != 64",
            10,
        ));
    }
    match ins.mnemonic {
        LCALL | LJMP => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[I16, M16, M32], true)
                .push(&[I16, I32], false)
                .forbidden(&[[MA, IA]])
                .check(ins)
        }

        JCXZ | JECXZ => ot_chk(ins, &[(&[I8], Optional::Needed)], &[], &[]),

        Mnemonic::CMOVA
        | Mnemonic::CMOVB
        | Mnemonic::CMOVC
        | Mnemonic::CMOVE
        | Mnemonic::CMOVG
        | Mnemonic::CMOVL
        | Mnemonic::CMOVO
        | Mnemonic::CMOVP
        | Mnemonic::CMOVS
        | Mnemonic::CMOVZ
        | Mnemonic::CMOVAE
        | Mnemonic::CMOVBE
        | Mnemonic::CMOVLE
        | Mnemonic::CMOVGE
        | Mnemonic::CMOVNA
        | Mnemonic::CMOVNB
        | Mnemonic::CMOVNC
        | Mnemonic::CMOVNE
        | Mnemonic::CMOVNG
        | Mnemonic::CMOVNL
        | Mnemonic::CMOVNO
        | Mnemonic::CMOVNP
        | Mnemonic::CMOVNS
        | Mnemonic::CMOVNZ
        | Mnemonic::CMOVPE
        | Mnemonic::CMOVPO
        | Mnemonic::CMOVNBE
        | Mnemonic::CMOVNLE
        | Mnemonic::CMOVNGE
        | Mnemonic::CMOVNAE => ot_chk(
            ins,
            &[
                (&[R16, R32], Optional::Needed),
                (&[R16, R32, M16, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),

        Mnemonic::PUSH => ot_chk(
            ins,
            &[(&[R16, R32, M16, M32, I8, I16, I32, SR], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::POP => ot_chk(
            ins,
            &[(&[R16, R32, M16, M32, DS, ES, SS, FS, GS], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::MOV => {
            use chkn::*;
            const CHK: CheckAPI<2> = const {
                CheckAPI::new()
                    .push(&[R8, R16, R32, R64, SR, CR, DR, M8, M16, M32], true)
                    .push(
                        &[R8, R16, R32, SR, CR, DR, M8, M16, M32, I8, I16, I32],
                        true,
                    )
                    .forbidden(&[
                        [MA, MA],
                        [SR, IA],
                        [CR, IA],
                        [DR, IA],
                        [DR, MA],
                        [MA, DR],
                        [DR, MA],
                        [MA, DR],
                        [SR, CR],
                        [SR, DR],
                        [CR, SR],
                        [CR, DR],
                        [DR, SR],
                        [SR, SR],
                        [DR, DR],
                        [CR, CR],
                    ])
                    .set_mode(CheckMode::X86)
            };
            CHK.check(ins)
        }
        XCHG => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, M8, M16, M32], Optional::Needed),
                (&[R8, R16, R32, M8, M16, M32], Optional::Needed),
            ],
            &[(MA, MA)],
            &[LOCK],
        ),
        CMP => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, M8, M16, M32], Optional::Needed),
                (
                    &[R8, R16, R32, M8, M16, M32, I8, I16, I32],
                    Optional::Needed,
                ),
            ],
            &[(MA, MA)],
            &[],
        ),
        Mnemonic::SUB
        | Mnemonic::ADD
        | Mnemonic::AND
        | Mnemonic::OR
        | Mnemonic::XOR
        | Mnemonic::ADC
        | SBB => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, M8, M16, M32], Optional::Needed),
                (
                    &[R8, R16, R32, M8, M16, M32, I8, I16, I32],
                    Optional::Needed,
                ),
            ],
            &[(MA, MA)],
            &[LOCK],
        ),
        Mnemonic::IMUL => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, M8, M16, M32], Optional::Needed),
                (&[R16, R32, M16, M32], Optional::Optional),
                (&[I8, I16, I32], Optional::Optional),
            ],
            &[(MA, MA)],
            &[],
        ),
        Mnemonic::SAL | Mnemonic::SHL | Mnemonic::SHR | Mnemonic::SAR | ROL | RCL | ROR | RCR => {
            ot_chk(
                ins,
                &[
                    (&[R8, R16, R32, M8, M16, M32], Optional::Needed),
                    (&[CL, I8], Optional::Needed),
                ],
                &[],
                &[],
            )
        }
        Mnemonic::TEST => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, M8, M16, M32], Optional::Needed),
                (&[I8, I16, I32, R8, R16, R32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::DIV | Mnemonic::IDIV | Mnemonic::MUL => chkn::CheckAPI::<1>::new()
            .push(&[R8, R16, R32, M8, M16, M32], true)
            .check(ins),
        Mnemonic::DEC | Mnemonic::INC | Mnemonic::NEG | Mnemonic::NOT => ot_chk(
            ins,
            &[(&[R8, R16, R32, M8, M16, M32], Optional::Needed)],
            &[],
            &[LOCK],
        ),

        Mnemonic::JMP | Mnemonic::CALL => ot_chk(
            ins,
            &[(&[I32, R32, R16, M32, M16], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::LEA => ot_chk(
            ins,
            &[(&[R16, R32], Optional::Needed), (&[MA], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::SYSCALL
        | Mnemonic::RET
        | Mnemonic::NOP
        | Mnemonic::POPF
        | Mnemonic::POPFD
        | Mnemonic::PUSHF
        | Mnemonic::PUSHFD => ot_chk(ins, &[], &[], &[]),
        BSF | BSR => ot_chk(
            ins,
            &[
                (&[R16, R32], Optional::Needed),
                (&[R16, R32, M16, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        BTC | BTR | BTS => ot_chk(
            ins,
            &[
                (&[R16, R32, M16, M32], Optional::Needed),
                (&[I8, R16, R32], Optional::Needed),
            ],
            &[],
            &[LOCK],
        ),
        Mnemonic::BT => ot_chk(
            ins,
            &[
                (&[R16, R32, M16, M32], Optional::Needed),
                (&[I8, R16, R32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        CBW | CMC | CWD | CDQ | CQO | CLD | CLI => ot_chk(ins, &[], &[], &[]),
        AAD | AAM => ot_chk(ins, &[(&[I8], Optional::Optional)], &[], &[]),

        // 32-bit only
        DAA | DAS | AAA | AAS => ot_chk(ins, &[], &[], &[]),

        // part b
        CWDE | CDQE | CLAC | CLTS | CLUI => ot_chk(ins, &[], &[], &[]),
        CLWB => ot_chk(ins, &[(&[M8], Optional::Needed)], &[], &[]),
        BLSI | ADCX | ADOX | BLSR | BLSMSK => ot_chk(
            ins,
            &[(&[R32], Optional::Needed), (&[R32, M32], Optional::Needed)],
            &[],
            &[],
        ),
        BSWAP => ot_chk(ins, &[(&[R32], Optional::Needed)], &[], &[]),
        ANDN | BZHI | BEXTR => ot_chk(
            ins,
            &[
                (&[R32], Optional::Needed),
                (&[R32], Optional::Needed),
                (&[R32, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        ARPL => ot_chk(
            ins,
            &[(&[R16, M16], Optional::Needed), (&[R16], Optional::Needed)],
            &[],
            &[],
        ),
        // part c
        CMPSTRB | CMPSTRW | CMPSTRD | SCASB | SCASW | SCASD => {
            ot_chk(ins, &[], &[], &[REPE, REPZ, REPNE, REPNZ])
        }
        ENDBR64 | ENDBR32 => ot_chk(ins, &[], &[], &[]),
        CMPXCHG => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, M8, M16, M32], Optional::Needed),
                (&[R8, R16, R32], Optional::Needed),
            ],
            &[],
            &[LOCK],
        ),
        CLDEMOTE => ot_chk(ins, &[(&[M8], Optional::Needed)], &[], &[]),
        CLRSSBSY => ot_chk(ins, &[(&[M64], Optional::Needed)], &[], &[]),
        CMPXCHG8B => ot_chk(ins, &[(&[M64], Optional::Needed)], &[], &[LOCK]),
        // part 2
        INTO => ot_chk(ins, &[], &[], &[]),
        INVPCID => ot_chk(
            ins,
            &[(&[R32], Optional::Needed), (&[M128], Optional::Needed)],
            &[],
            &[],
        ),
        // part 3
        MULX | PEXT | PDEP => ot_chk(
            ins,
            &[
                (&[R32], Optional::Needed),
                (&[R32], Optional::Needed),
                (&[R32, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        MOVZX => ot_chk(
            ins,
            &[
                (&[R16, R32], Optional::Needed),
                (&[R8, R16, M8, M16], Optional::Needed),
            ],
            &[(R16, M16), (R16, M16)],
            &[],
        ),
        MOVSTRB | MOVSTRW | MOVSTRD => ot_chk(ins, &[], &[], &[REP]),
        MOVDIRI => ot_chk(
            ins,
            &[(&[M32], Optional::Needed), (&[R32], Optional::Needed)],
            &[],
            &[],
        ),
        MOVBE => ot_chk(
            ins,
            &[
                (&[R16, R32, M16, M32], Optional::Needed),
                (&[R16, R32, M16, M32], Optional::Needed),
            ],
            &[(RA, RA), (MA, MA)],
            &[],
        ),
        LZCNT => ot_chk(
            ins,
            &[
                (&[R16, R32], Optional::Needed),
                (&[R16, M16, R32, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),

        RDRAND | RDSEED => ot_chk(ins, &[(&[R16, R32], Optional::Needed)], &[], &[]),
        RDPID => ot_chk(ins, &[(&[R32], Optional::Needed)], &[], &[]),
        RDSSPD => ot_chk(ins, &[(&[R32], Optional::Needed)], &[], &[]),
        RDSSPQ => ot_chk(ins, &[(&[R64], Optional::Needed)], &[], &[]),
        RORX => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[R32, M32], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        SHLX | SHRX | SARX => ot_chk(
            ins,
            &[
                (&[R32], Optional::Needed),
                (&[R32, M32], Optional::Needed),
                (&[R32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        SHLD | SHRD => ot_chk(
            ins,
            &[
                (&[R16, M16, R32, M32], Optional::Needed),
                (&[R16, R32], Optional::Needed),
                (&[I8, CL], Optional::Needed),
            ],
            &[],
            &[],
        ),
        XADD => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, M8, M16, M32], Optional::Needed),
                (&[R8, R16, R32], Optional::Needed),
            ],
            &[],
            &[LOCK],
        ),

        // #   #  #   #  #   #
        // ## ##  ## ##   # #
        // # # #  # # #    #
        // #   #  #   #   # #
        // #   #  #   #  #   #
        // (MMX/SSE2)
        Mnemonic::MOVD => ot_chk(
            ins,
            &[
                (&[MMX, XMM, R32, M32], Optional::Needed),
                (&[MMX, XMM, R32, M32], Optional::Needed),
            ],
            &[(M32, M32), (R32, R32), (MMX, MMX), (XMM, MMX), (MMX, XMM)],
            &[],
        ),
        Mnemonic::MOVQ | Mnemonic::PUSHAQ | MOVSTRQ | SCASQ | STOSQ | Mnemonic::POPAQ | INCSSPQ => {
            let er = Error::new(
                "you tried to use instruction that is invalid when bits != 64",
                10,
            );
            Err(er)
        }
        Mnemonic::PTWRITE => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[R32, M32], true).check(ins)
        }
        Mnemonic::WBNOINVD | Mnemonic::PCONFIG => {
            use chkn::*;
            CheckAPI::<0>::new().check(ins)
        }
        Mnemonic::MOVSX => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R16, R32], true)
                .push(&[R8, R16, M8, M16], true)
                .set_mode(CheckMode::NOSIZE)
                .check(ins)
        }
        _ => shr_chk(ins),
    }
}

pub fn check_ins64bit(ins: &Instruction) -> Result<(), Error> {
    use Mnemonic::*;
    match ins.mnemonic {
        LCALL | LJMP => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[I16, M16, M32, M64], true)
                .push(&[I16, I32], false)
                .forbidden(&[[MA, IA]])
                .check(ins)
        }
        Mnemonic::RDMSRLIST
        | Mnemonic::PBNDKB
        | Mnemonic::WRMSRLIST
        | Mnemonic::WBNOINVD
        | Mnemonic::PCONFIG
        | Mnemonic::POPAQ => {
            use chkn::*;
            CheckAPI::<0>::new().check(ins)
        }
        AWRSSQ | AWRUSSQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[M64], true)
                .push(&[R64], true)
                .apx(APXVariant::LegacyExtension, false)
                .check(ins)
        }
        AWRSSD | AWRUSSD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[M32], true)
                .push(&[R32], true)
                .apx(APXVariant::LegacyExtension, false)
                .check(ins)
        }
        Mnemonic::PREFETCHIT0 | Mnemonic::PREFETCHIT1 => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[M8], true).check(ins)
        }
        Mnemonic::PTWRITE => {
            use chkn::*;
            CheckAPI::<1>::new()
                .push(&[R32, R64, M32, M64], true)
                .check(ins)
        }
        Mnemonic::ENQCMDS | Mnemonic::MOVDIR64B | Mnemonic::ENQCMD | Mnemonic::AENQCMD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R16, R32, R64], true)
                .push(&[M512], true)
                .check(ins)
        }
        Mnemonic::RDFSBASE | Mnemonic::RDGSBASE => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[R32, R64], true).check(ins)
        }
        Mnemonic::MOVSX => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R16, R32, R64], true)
                .push(&[R8, R16, M8, M16], true)
                .set_mode(CheckMode::NOSIZE)
                .check(ins)
        }
        Mnemonic::MOVSXD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R16, R32, R64], true)
                .push(&[R16, R32, M16, M32], true)
                .set_mode(CheckMode::NOSIZE)
                .check(ins)
        }

        ACMPBXADD | ACMPBEXADD | ACMPLXADD | ACMPLEXADD | ACMPNBXADD | ACMPZXADD | ACMPNBEXADD
        | ACMPNLXADD | ACMPNLEXADD | ACMPNOXADD | ACMPNSXADD | ACMPNZXADD | ACMPOXADD
        | ACMPSXADD | ACMPNAXADD | ACMPAXADD | ACMPNAEXADD | ACMPAEXADD | ACMPEXADD
        | ACMPNEXADD | ACMPGEXADD | ACMPGXADD | ACMPNGXADD | ACMPNGEXADD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[M32, M64], true)
                .push(&[R32, R64], true)
                .push(&[R32, R64], true)
                .apx(APXVariant::VexExtension, false)
                .check(ins)
        }
        CMPBXADD | CMPBEXADD | CMPLXADD | CMPLEXADD | CMPNBXADD | CMPZXADD | CMPNBEXADD
        | CMPNLXADD | CMPNLEXADD | CMPNOXADD | CMPNSXADD | CMPNZXADD | CMPOXADD | CMPSXADD
        | CMPNAXADD | CMPAXADD | CMPNAEXADD | CMPAEXADD | CMPEXADD | CMPNEXADD | CMPGEXADD
        | CMPGXADD | CMPNGXADD | CMPNGEXADD | CMPCXADD | CMPNCXADD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[M32, M64], true)
                .push(&[R32, R64], true)
                .push(&[R32, R64], true)
                .check(ins)
        }

        // APX
        Mnemonic::PUSHP => {
            use chkn::*;
            CheckAPI::<1>::new()
                .push(&[R64], true)
                .apx(APXVariant::Rex2, false)
                .check(ins)
        }
        Mnemonic::PUSH2 | Mnemonic::PUSH2P => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R64], true)
                .push(&[R64], true)
                .apx(APXVariant::LegacyExtension, false)
                .check(ins)
        }
        Mnemonic::POPP => {
            use chkn::*;
            CheckAPI::<1>::new()
                .push(&[R64], true)
                .apx(APXVariant::Rex2, false)
                .check(ins)
        }
        Mnemonic::POP2 | Mnemonic::POP2P => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R64], true)
                .push(&[R64], true)
                .apx(APXVariant::LegacyExtension, false)
                .check(ins)
        }

        Mnemonic::CFCMOVA
        | Mnemonic::CFCMOVB
        | Mnemonic::CFCMOVE
        | Mnemonic::CFCMOVG
        | Mnemonic::CFCMOVL
        | Mnemonic::CFCMOVO
        | Mnemonic::CFCMOVP
        | Mnemonic::CFCMOVS
        | Mnemonic::CFCMOVZ
        | Mnemonic::CFCMOVAE
        | Mnemonic::CFCMOVBE
        | Mnemonic::CFCMOVLE
        | Mnemonic::CFCMOVGE
        | Mnemonic::CFCMOVNA
        | Mnemonic::CFCMOVNB
        | Mnemonic::CFCMOVNE
        | Mnemonic::CFCMOVNC
        | Mnemonic::CFCMOVC
        | Mnemonic::CFCMOVNG
        | Mnemonic::CFCMOVNL
        | Mnemonic::CFCMOVNO
        | Mnemonic::CFCMOVNP
        | Mnemonic::CFCMOVNS
        | Mnemonic::CFCMOVNZ
        | Mnemonic::CFCMOVPE
        | Mnemonic::CFCMOVPO
        | Mnemonic::CFCMOVNBE
        | Mnemonic::CFCMOVNLE
        | Mnemonic::CFCMOVNGE
        | Mnemonic::CFCMOVNAE => chkn::CheckAPI::<3>::new()
            .push(&[R16, R32, R64, M16, M32, M64], true)
            .push(&[R16, R32, R64, M16, M32, M64], true)
            .push(&[R16, R32, R64, M16, M32, M64], false)
            .forbidden(&[
                [MA, MA, MA],
                [RA, MA, MA],
                [MA, RA, MA],
                [MA, MA, RA],
                [RA, RA, MA],
                [RA, MA, RA],
                [MA, RA, RA],
                [MA, RA, RA],
                [RA, MA, RA],
            ])
            .apx(APXVariant::LegacyExtension, true)
            .check(ins),
        SETB | SETBE | SETL | SETLE | SETNB | SETNBE | SETNL | SETNLE | SETNO | SETNS | SETNZ
        | SETZ | SETO | SETS | SETA | SETNA | SETAE | SETNAE | SETE | SETNE | SETGE | SETG
        | SETNG | SETNGE | SETC | SETNC => {
            use chkn::*;
            CheckAPI::<1>::new()
                .push(&[RA, MA], true)
                .apx(APXVariant::LegacyExtension, false)
                .check(ins)
        }
        JMPABS => {
            use chkn::*;
            CheckAPI::<1>::new()
                .push(&[I64], true)
                .apx(APXVariant::Rex2, false)
                .check(ins)
        }
        AMOVRS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[MA], true)
                .push(&[RA], true)
                .apx(APXVariant::LegacyExtension, false)
                .check(ins)
        }
        AAADD | AAAND | AAOR | AAXOR | AMOVDIRI => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[M32, M64], true)
                .push(&[R32, R64], true)
                .apx(APXVariant::LegacyExtension, false)
                .check(ins)
        }
        AADCX | AADOX | ASHLX => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[R32, R64, M32, M64], true)
                .push(&[R32, R64, M32, M64], true)
                .push(&[R32, R64, M32, M64], false)
                .forbidden(&[[MA, RA, MA], [MA, MA, MA], [MA, MA, RA], [RA, MA, MA]])
                .apx(APXVariant::LegacyExtension, false)
                .check(ins)
        }
        ABEXTR | ABZHI => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[R32, R64], true)
                .push(&[R32, R64, M32, M64], true)
                .push(&[R32, R64], true)
                .apx(APXVariant::VexExtension, true)
                .check(ins)
        }
        AIMUL => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .forbidden(&[
                    [MA, MA, MA],
                    [MA, RA, MA],
                    [MA, RA, RA],
                    [RA, MA, RA],
                    [MA, MA, RA],
                    [RA, MA, MA],
                    [R8, ANY, ANY],
                    [M8, ANY, ANY],
                ])
                .apx(APXVariant::LegacyExtension, true)
                .check(ins)
        }
        ASHLD => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[R16, R32, R64, M16, M32, M64], true)
                .push(&[R16, R32, R64, M16, M32, M64, I8], true)
                .push(&[R16, R32, R64, M16, M32, M64, I8], true)
                .push(&[CL, I8], false)
                .forbidden(&[
                    [MA, MA, MA, ANY],
                    [RA, I8, I8, ANY],
                    [MA, I8, I8, ANY],
                    [MA, RA, MA, ANY],
                    [MA, RA, MA, ANY],
                ])
                .apx(APXVariant::LegacyExtension, true)
                .check(ins)
        }
        ARCL | ARCR | AROL | AROR | ASAR | ASHL | ASHR => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[RA, MA], true)
                .push(&[RA, MA, I8], true)
                .push(&[CL, I8], false)
                .forbidden(&[[MA, MA, ANY], [RA, I8, ANY], [MA, I8, ANY], [MA, RA, ANY]])
                .apx(APXVariant::LegacyExtension, true)
                .check(ins)
        }
        ARORX | ASARX => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[R32, R64], true)
                .push(&[R32, R64, M32, M64], true)
                .push(&[I8], true)
                .apx(APXVariant::LegacyExtension, false)
                .check(ins)
        }
        ARDMSR => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R64], true)
                .push(&[I32], true)
                .apx(APXVariant::LegacyExtension, false)
                .check(ins)
        }
        APOPCNT => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R8, R16, R32, R64], true)
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .apx(APXVariant::LegacyExtension, false)
                .check(ins)
        }
        AIMULZU => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[R8, R16, R32, R64], true)
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .push(&[I8, I16, I32], true)
                .apx(APXVariant::LegacyExtension, true)
                .check(ins)
        }
        ABLSI | ABLSMSK | ABLSR => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R32, R64], true)
                .push(&[R32, R64, M32, M64], true)
                .apx(APXVariant::VexExtension, true)
                .check(ins)
        }
        ADIV | AIDIV | AMUL => {
            use chkn::*;
            CheckAPI::<1>::new()
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .apx(APXVariant::LegacyExtension, true)
                .check(ins)
        }
        AANDN | AMULX | APDEP | APEXT => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[R32, R64], true)
                .push(&[R32, R64], true)
                .push(&[R32, R64, M32, M64], true)
                .apx(APXVariant::LegacyExtension, true)
                .check(ins)
        }
        AMOVBE => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], false)
                .forbidden(&[[MA, MA]])
                .apx(APXVariant::LegacyExtension, false)
                .check(ins)
        }
        ADEC | AINC | ANEG | ANOT => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], false)
                .forbidden(&[[MA, MA]])
                .apx(APXVariant::LegacyExtension, true)
                .check(ins)
        }
        ALZCNT | ATZCNT => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R8, R16, R32, R64], true)
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .apx(APXVariant::LegacyExtension, true)
                .check(ins)
        }
        ACRC32 => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R8, R16, R32, R64], true)
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .apx(APXVariant::LegacyExtension, false)
                .check(ins)
        }
        AADC | AADD | AOR | AXOR | AAND | ASBB | ASUB => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64, I8, I16, I32], true)
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64, I8, I16, I32], false)
                .forbidden(&[
                    [MA, RA, MA],
                    [MA, IA, IA],
                    [RA, IA, IA],
                    [MA, IA, MA],
                    [MA, MA, MA],
                    [MA, MA, RA],
                ])
                .apx(APXVariant::LegacyExtension, true)
                .check(ins)
        }
        AINVEPT | AINVPCID | AINVVPID => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R64], true)
                .push(&[M128], true)
                .set_mode(CheckMode::NOSIZE)
                .apx(APXVariant::LegacyExtension, false)
                .check(ins)
        }

        JRCXZ | JECXZ => ot_chk(ins, &[(&[I8], Optional::Needed)], &[], &[]),

        Mnemonic::CMOVA
        | Mnemonic::CMOVB
        | Mnemonic::CMOVC
        | Mnemonic::CMOVE
        | Mnemonic::CMOVG
        | Mnemonic::CMOVL
        | Mnemonic::CMOVO
        | Mnemonic::CMOVP
        | Mnemonic::CMOVS
        | Mnemonic::CMOVZ
        | Mnemonic::CMOVAE
        | Mnemonic::CMOVBE
        | Mnemonic::CMOVLE
        | Mnemonic::CMOVGE
        | Mnemonic::CMOVNA
        | Mnemonic::CMOVNB
        | Mnemonic::CMOVNC
        | Mnemonic::CMOVNE
        | Mnemonic::CMOVNG
        | Mnemonic::CMOVNL
        | Mnemonic::CMOVNO
        | Mnemonic::CMOVNP
        | Mnemonic::CMOVNS
        | Mnemonic::CMOVNZ
        | Mnemonic::CMOVPE
        | Mnemonic::CMOVPO
        | Mnemonic::CMOVNBE
        | Mnemonic::CMOVNLE
        | Mnemonic::CMOVNGE
        | Mnemonic::CMOVNAE => chkn::CheckAPI::<2>::new()
            .push(&[R16, R32, R64], true)
            .push(&[R16, R32, R64, M16, M32, M64], true)
            .check(ins),
        Mnemonic::CLFLUSH => ot_chk(ins, &[(&[M8], Optional::Needed)], &[], &[]),
        Mnemonic::PAUSE | Mnemonic::LFENCE | Mnemonic::MFENCE => ot_chk(ins, &[], &[], &[]),
        Mnemonic::PUSH => ot_chk(
            ins,
            &[(
                &[R16, R64, M16, M64, I8, I16, I32, FS, GS],
                Optional::Needed,
            )],
            &[],
            &[],
        ),
        Mnemonic::POP => ot_chk(
            ins,
            &[(&[R16, R64, M16, M64, FS, GS], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::MOV => {
            use chkn::*;
            CheckAPI::new()
                .push(&[R8, R16, R32, R64, SR, CR, DR, M8, M16, M32, M64], true)
                .push(
                    &[
                        R8, R16, R32, R64, SR, CR, DR, M8, M16, M32, M64, I8, I16, I32, I64,
                    ],
                    true,
                )
                .forbidden(&[
                    [MA, MA],
                    [SR, IA],
                    [CR, IA],
                    [CR, R32],
                    [DR, IA],
                    [DR, R32],
                    [DR, MA],
                    [MA, DR],
                    [DR, MA],
                    [MA, DR],
                    [SR, CR],
                    [SR, DR],
                    [CR, SR],
                    [CR, DR],
                    [DR, SR],
                    [SR, SR],
                    [DR, DR],
                    [CR, CR],
                ])
                .set_mode(CheckMode::X86)
                .check(ins)
        }
        XCHG => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .push(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .forbidden(&[[MA, MA]])
                .additional_mnemonics(&[LOCK])
                .set_mode(CheckMode::X86)
                .check(ins)
        }
        Mnemonic::CMP => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed),
                (
                    &[R8, R16, R32, R64, M8, M16, M32, M64, I8, I16, I32],
                    Optional::Needed,
                ),
            ],
            &[(MA, MA)],
            &[],
        ),
        Mnemonic::SUB
        | Mnemonic::ADD
        | Mnemonic::AND
        | Mnemonic::OR
        | Mnemonic::XOR
        | ADC
        | SBB => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed),
                (
                    &[R8, R16, R32, R64, M8, M16, M32, M64, I8, I16, I32],
                    Optional::Needed,
                ),
            ],
            &[(MA, MA)],
            &[LOCK],
        ),
        Mnemonic::IMUL => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed),
                (&[R16, R32, R64, M16, M32, M64], Optional::Optional),
                (&[I8, I16, I32], Optional::Optional),
            ],
            &[(MA, MA)],
            &[],
        ),
        Mnemonic::SAL | Mnemonic::SHL | Mnemonic::SHR | Mnemonic::SAR | ROL | RCL | ROR | RCR => {
            ot_chk(
                ins,
                &[
                    (&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed),
                    (&[CL, I8], Optional::Needed),
                ],
                &[],
                &[],
            )
        }
        Mnemonic::TEST => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed),
                (&[I8, I16, I32, R8, R16, R32, R64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::DIV | Mnemonic::IDIV | Mnemonic::MUL => ot_chk(
            ins,
            &[(&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::DEC | Mnemonic::INC | Mnemonic::NEG | Mnemonic::NOT => ot_chk(
            ins,
            &[(&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed)],
            &[],
            &[LOCK],
        ),
        Mnemonic::JMP | Mnemonic::CALL => {
            ot_chk(ins, &[(&[I32, R64, M64], Optional::Needed)], &[], &[])
        }
        Mnemonic::LEA => ot_chk(
            ins,
            &[
                (&[R16, R32, R64], Optional::Needed),
                (&[MA], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::SYSCALL
        | Mnemonic::RET
        | Mnemonic::NOP
        | Mnemonic::PUSHF
        | Mnemonic::POPF
        | Mnemonic::POPFQ
        | Mnemonic::PUSHFQ => ot_chk(ins, &[], &[], &[]),
        BSF | BSR => ot_chk(
            ins,
            &[
                (&[R16, R32, R64], Optional::Needed),
                (&[R16, R32, M16, M32, M64, R64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        BT => ot_chk(
            ins,
            &[
                (&[R16, R32, R64, M16, M32, M64], Optional::Needed),
                (&[I8, R16, R32, R64], Optional::Needed),
            ],
            &[],
            &[],
        ),

        BTC | BTR | BTS => ot_chk(
            ins,
            &[
                (&[R16, R32, R64, M16, M32, M64], Optional::Needed),
                (&[I8, R16, R32, R64], Optional::Needed),
            ],
            &[],
            &[LOCK],
        ),
        CBW | CMC | CWD | CDQ | CQO | CLD | CLI => ot_chk(ins, &[], &[], &[]),

        // part b
        CWDE | CDQE | CLAC | CLTS | CLUI => ot_chk(ins, &[], &[], &[]),
        CLWB => ot_chk(ins, &[(&[M8], Optional::Needed)], &[], &[]),
        BLSI | ADCX | ADOX | BLSR | BLSMSK => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[R32, M32, R64, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        BSWAP => ot_chk(ins, &[(&[R32, R64], Optional::Needed)], &[], &[]),
        ANDN | BZHI | BEXTR => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[R32, R64], Optional::Needed),
                (&[R32, M32, R64, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        // part c
        CMPSTRB | CMPSTRW | CMPSTRD | CMPSTRQ | SCASB | SCASW | SCASD | SCASQ => {
            ot_chk(ins, &[], &[], &[REPE, REPZ, REPNE, REPNZ])
        }
        ENDBR64 | ENDBR32 => ot_chk(ins, &[], &[], &[]),
        CMPXCHG => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, M8, M16, M32, R64, M64], Optional::Needed),
                (&[R8, R16, R32, R64], Optional::Needed),
            ],
            &[],
            &[LOCK],
        ),
        CLDEMOTE => ot_chk(ins, &[(&[M8], Optional::Needed)], &[], &[]),
        CLRSSBSY => ot_chk(ins, &[(&[M64], Optional::Needed)], &[], &[]),
        CMPXCHG8B => ot_chk(ins, &[(&[M64], Optional::Needed)], &[], &[LOCK]),
        CMPXCHG16B => ot_chk(ins, &[(&[M128], Optional::Needed)], &[], &[LOCK]),
        // part 2
        INVPCID => ot_chk(
            ins,
            &[(&[R64], Optional::Needed), (&[M128], Optional::Needed)],
            &[],
            &[],
        ),
        IRETQ | LODSQ => ot_chk(ins, &[], &[], &[]),

        // part 3
        MULX | PEXT | PDEP => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[R32, R64], Optional::Needed),
                (&[R32, R64, M32, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        MOVZX => ot_chk(
            ins,
            &[
                (&[R16, R32, R64], Optional::Needed),
                (&[R8, R16, M8, M16], Optional::Needed),
            ],
            &[(R16, M16), (R16, R16)],
            &[],
        ),
        MOVSTRB | MOVSTRW | MOVSTRD | MOVSTRQ => ot_chk(ins, &[], &[], &[REP]),
        MOVDIRI => ot_chk(
            ins,
            &[
                (&[M32, M64], Optional::Needed),
                (&[R32, R64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        MOVBE => ot_chk(
            ins,
            &[
                (&[R16, R32, R64, M16, M32, M64], Optional::Needed),
                (&[R16, R32, R64, M16, M32, M64], Optional::Needed),
            ],
            &[(RA, RA), (MA, MA)],
            &[],
        ),
        LZCNT => ot_chk(
            ins,
            &[
                (&[R16, R32, R64], Optional::Needed),
                (&[R16, M16, R32, M32, R64, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        // part 4
        SENDUIPI => ot_chk(ins, &[(&[R64], Optional::Needed)], &[], &[]),
        RDRAND | RDSEED => ot_chk(ins, &[(&[R16, R32, R64], Optional::Needed)], &[], &[]),
        RDSSPD => ot_chk(ins, &[(&[R32], Optional::Needed)], &[], &[]),
        RDSSPQ => ot_chk(ins, &[(&[R64], Optional::Needed)], &[], &[]),
        RORX => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[R32, R64, M32, M64], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        SHLX | SHRX | SARX => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[R32, R64, M32, M64], Optional::Needed),
                (&[R32, R64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        RDPID => ot_chk(ins, &[(&[R64], Optional::Needed)], &[], &[]),
        WRFSBASE | WRGSBASE => ot_chk(ins, &[(&[R32, R64], Optional::Needed)], &[], &[]),
        SHLD | SHRD => ot_chk(
            ins,
            &[
                (&[R16, M16, R32, M32, R64, M64], Optional::Needed),
                (&[R16, R32, R64], Optional::Needed),
                (&[I8, CL], Optional::Needed),
            ],
            &[],
            &[],
        ),
        XADD => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed),
                (&[R8, R16, R32, R64], Optional::Needed),
            ],
            &[],
            &[LOCK],
        ),
        WRSSQ | WRUSSQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[M64], true)
                .push(&[R64], true)
                .check(ins)
        }
        INCSSPQ => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[R64], true).check(ins)
        }
        _ => shr_chk(ins),
    }
}

pub fn shr_chk(ins: &Instruction) -> Result<(), Error> {
    use Mnemonic::*;
    match ins.mnemonic {
        INVLPGA | VMRUN | VMLOAD | VMSAVE | STGI | CLGI | VMMCALL | SKINIT | VMGEXIT | PSMASH
        | RMPUPDATE | PVALIDATE | RMPADJUST | RMPQUERY | RMPREAD | VMXOFF | VMLAUNCH | VMRESUME
        | VMCALL | VMFUNC | SEAMOPS | SEAMRET | SEAMCALL | TDCALL | FNINIT | FINIT | FNCLEX
        | FCLEX | FNDISI | FDISI | FENI | FNENI | FLDZ | FLD1 | FLDPI | FLDL2T | FLDL2E
        | FLDLG2 | FLDLN2 | FCOMPP | FCHS | FABS | FTST | FXAM | FXTRACT | FRNDINT | FPREM
        | FSQRT | FSCALE | F2XM1 | FYL2X | FPTAN | FYL2XP1 | FDECSTP | FINCSTP | FNSETPM
        | FSETPM | FNSTSWAX | FSTSWAX | FUCOMPP | FPREM1 | FSINCOS | FSIN | FCOS | ENCLV
        | ENCLS | ENCLU | SAVEPREVSSP | FPATAN => {
            use chkn::*;
            CheckAPI::<0>::new().check(ins)
        }
        FCMOVB | FCMOVE | FCMOVBE | FCMOVU | FCMOVNB | FCMOVNE | FCMOVNBE | FCMOVNU | FADDP
        | FMULP | FSUBRP | FFREE | FSUBP | FDIVP | FDIVRP | FCOMP | FCOMI | FCOMIP | FUCOMI
        | FUCOMIP => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[ST], true).check(ins)
        }
        FIADD | FIMUL | FISUBR | FISUB | FIDIV | FIDIVR | FICOMP | FICOM => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[M32, M64], true).check(ins)
        }
        FADD | FMUL | FSUBR | FSUB | FDIV | FDIVR => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[M32, M64, ST], true)
                .push(&[ST], false)
                .forbidden(&[[MA, ST]])
                .check(ins)
        }
        CLRSSBSY => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[M64], true).check(ins)
        }
        WRSSD | WRUSSD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[M32], true)
                .push(&[R32], true)
                .check(ins)
        }
        FILD | FISTP => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[M16, M32, M64], true).check(ins)
        }
        FIST => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[M16, M32], true).check(ins)
        }
        FBLD | FBSTP => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[M80], true).check(ins)
        }
        FXCH | FUCOM | FUCOMP => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[ST], true).check(ins)
        }
        FST | FCOM => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[M32, M64, ST], true).check(ins)
        }
        FLD | FSTP => {
            use chkn::*;
            CheckAPI::<1>::new()
                .push(&[M32, M64, M80, ST], true)
                .check(ins)
        }
        FNSTCW | FSTCW | FNSTSW | FSTSW | FLDCW | FLDENV | FNSTENV | FNSAVE | FSAVE | FSTENV
        | FRSTOR | FXSAVE | FXRSTOR | FXSAVE64 | FXRSTOR64 => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[MA], true).check(ins)
        }
        INVEPT | INVVPID => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R16, R32, R64], true)
                .push(&[M128], true)
                .check(ins)
        }
        AESENC256KL | AESDEC256KL => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM], true)
                .push(&[M512], true)
                .set_mode(CheckMode::NOSIZE)
                .check(ins)
        }
        AESENCWIDE128KL | AESDECWIDE128KL => {
            use chkn::*;
            CheckAPI::<1>::new()
                .push(&[MA], true)
                .set_mode(CheckMode::NOSIZE)
                .check(ins)
        }
        AESENC128KL | AESDEC128KL => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM], true)
                .push(&[MA], true)
                .set_mode(CheckMode::NOSIZE)
                .check(ins)
        }
        ENCODEKEY128 | ENCODEKEY256 => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R32], true)
                .push(&[R32], true)
                .check(ins)
        }
        LOADIWKEY => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .check(ins)
        }
        INCSSPD => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[R32], true).check(ins)
        }
        VMWRITE => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R64], true)
                .push(&[R64, M64], true)
                .check(ins)
        }
        VMREAD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R64, M64], true)
                .push(&[R64], true)
                .check(ins)
        }
        VMXON | VMPTRLD | VMPTRST | VMCLEAR => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[M64], true).check(ins)
        }

        LGDT | LIDT | Mnemonic::SGDT | Mnemonic::SIDT => {
            ot_chk(ins, &[(&[M16], Optional::Needed)], &[], &[])
        }

        LFS | LGS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R16, R32], true)
                .push(&[M16, M32], true)
                .check(ins)
        }

        OUT => ot_chk(
            ins,
            &[
                (&[DX, I8], Optional::Needed),
                (&[AL, AX, EAX], Optional::Needed),
            ],
            &[],
            &[],
        ),
        IN => ot_chk(
            ins,
            &[
                (&[AL, AX, EAX], Optional::Needed),
                (&[DX, I8], Optional::Needed),
            ],
            &[],
            &[],
        ),

        // instruction as "variable"
        BYTELE | BYTEBE => ot_chk(ins, &[(&[I8], Optional::Needed)], &[], &[]),
        WORDLE | WORDBE => ot_chk(ins, &[(&[I8, I16], Optional::Needed)], &[], &[]),
        DWORDLE | DWORDBE => ot_chk(ins, &[(&[I8, I16, I32], Optional::Needed)], &[], &[]),
        QWORDLE | QWORDBE => ot_chk(ins, &[(&[I8, I16, I32, I64], Optional::Needed)], &[], &[]),
        EMPTY => ot_chk(ins, &[(&[I8, I16], Optional::Needed)], &[], &[]),
        STRING | ASCII => ot_chk(
            ins,
            &[(&[crate::shr::atype::STRING], Optional::Needed)],
            &[],
            &[],
        ),

        LTR => ot_chk(ins, &[(&[R16, M16], Optional::Needed)], &[], &[]),
        PREFETCHW | PREFETCH0 | PREFETCHIT0 | PREFETCHIT1 | PREFETCH1 | PREFETCH2 | PREFETCHA => {
            ot_chk(ins, &[(&[M8], Optional::Needed)], &[], &[])
        }
        LSL => ot_chk(
            ins,
            &[
                (&[R16, R32, R64], Optional::Needed),
                (&[R16, M16, R32, M32, R64, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        OUTSB | OUTSW | OUTSD | STOSB | STOSW | STOSD | STOSQ => ot_chk(ins, &[], &[], &[REP]),

        SFENCE | STAC | STC | STD | STI | STUI | SYSENTER | SYSEXIT | SYSRET | TESTUI | UD2
        | POPAW | POPAD | POPAQ | PUSHAW | PUSHAD | PUSHAQ | UIRET | WAIT | FWAIT | WBINVD
        | WRMSR | WRPKRU => ot_chk(ins, &[], &[], &[]),
        TPAUSE | UMWAIT => ot_chk(
            ins,
            &[
                (&[R32], Optional::Needed),
                (&[EDX], Optional::Optional),
                (&[EAX], Optional::Needed),
            ],
            &[],
            &[],
        ),
        UD0 | UD1 => ot_chk(
            ins,
            &[(&[R32], Optional::Needed), (&[R32, M32], Optional::Needed)],
            &[],
            &[],
        ),
        UMONITOR => ot_chk(ins, &[(&[R16, R32, R64], Optional::Needed)], &[], &[]),
        SMSW => ot_chk(ins, &[(&[R16, R32, R64, M16], Optional::Needed)], &[], &[]),
        STR | VERR | VERW => ot_chk(ins, &[(&[R16, M16], Optional::Needed)], &[], &[]),
        // rm 64-bit
        SHLD | SHRD => ot_chk(
            ins,
            &[
                (&[R16, M16, R32, M32, R64, M64], Optional::Needed),
                (&[R16, R32, R64], Optional::Needed),
                (&[I8, CL], Optional::Needed),
            ],
            &[],
            &[],
        ),

        LOOP
        | LOOPNE
        | LOOPE
        | Mnemonic::JA
        | Mnemonic::JB
        | Mnemonic::JC
        | Mnemonic::JO
        | Mnemonic::JP
        | Mnemonic::JS
        | Mnemonic::JAE
        | Mnemonic::JNAE
        | Mnemonic::JNBE
        | Mnemonic::JNGE
        | Mnemonic::JBE
        | Mnemonic::JNO
        | Mnemonic::JNP
        | Mnemonic::JPO
        | Mnemonic::JPE
        | Mnemonic::JNA
        | Mnemonic::JNL
        | Mnemonic::JNLE
        | Mnemonic::JNC
        | Mnemonic::JNB
        | Mnemonic::JNG
        | Mnemonic::JE
        | Mnemonic::JNE
        | Mnemonic::JZ
        | Mnemonic::JNZ
        | Mnemonic::JL
        | Mnemonic::JLE
        | Mnemonic::JG
        | Mnemonic::JGE => ot_chk(ins, &[(&[I32, I16, I8], Optional::Needed)], &[], &[]),
        Mnemonic::CPUID => ot_chk(ins, &[], &[], &[]),

        ENTER => ot_chk(
            ins,
            &[(&[I8, I16], Optional::Needed), (&[I8], Optional::Needed)],
            &[],
            &[],
        ),
        HLT | INT3 | INT1 | IRET | IRETD | LAHF | LEAVE => ot_chk(ins, &[], &[], &[]),

        INSB | INSW | INSD | LODSB | LODSW | LODSD => ot_chk(ins, &[], &[], &[REP]),

        HRESET => ot_chk(ins, &[(&[I8], Optional::Needed)], &[], &[]),
        INVD | INVLPG => ot_chk(ins, &[], &[], &[]),
        INT => ot_chk(ins, &[(&[I8], Optional::Needed)], &[], &[]),
        LAR => ot_chk(
            ins,
            &[
                (&[R16], Optional::Needed),
                (&[R16, M16, R32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        LLDT | SLDT | LMSW => ot_chk(ins, &[(&[R16, M16], Optional::Needed)], &[], &[]),

        RDMSR | RDPKRU | RDPMC | RDTSC | RDTSCP | RSM | SAHF | SERIALIZE | SETSSBY => {
            ot_chk(ins, &[], &[], &[])
        }
        RSTORSSP => ot_chk(ins, &[(&[M64], Optional::Needed)], &[], &[]),

        SETA | SETAE | SETB | SETBE | SETC | SETE | SETG | SETGE | SETL | SETLE | SETNA
        | SETNAE | SETNB | SETNBE | SETNC | SETNE | SETNG | SETNL | SETNGE | SETNLE | SETNO
        | SETNP | SETNS | SETNZ | SETO | SETP | SETPE | SETPO | SETS | SETZ => ot_chk(
            ins,
            &[(&[crate::shr::atype::R8, M8], Optional::Needed)],
            &[],
            &[],
        ),

        // norm-part6
        XABORT => ot_chk(ins, &[(&[I8], Optional::Needed)], &[], &[]),
        XACQUIRE | XRELEASE | XEND | XGETBV | XLAT | XLATB | XLATB64 | XRESLDTRK | XSETBV
        | NOPL | SWAPGS | XSUSLDTRK | XTEST => ot_chk(ins, &[], &[], &[]),

        XBEGIN => ot_chk(ins, &[(&[I32, I16], Optional::Needed)], &[], &[]),
        XRSTOR | XRSTORS | XRSTORS64 | XSAVE | XSAVE64 | XSAVEC | XSAVEC64 | XSAVEOPT
        | XSAVEOPT64 | XSAVES | XSAVES64 | XRSTOR64 => {
            ot_chk(ins, &[(&[M32, M64], Optional::Needed)], &[], &[])
        }

        // sha
        SHA1MSG1 | SHA1MSG2 | SHA1NEXTE | SHA256MSG1 | SHA256MSG2 | SHA256RNDS2 => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        SHA1RNDS4 => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),

        // #####  #####  #####
        // #      #      #
        // #####  #####  #####
        //     #      #  #
        // #####  #####  #####
        // (SSE)
        Mnemonic::CMPSS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::UNPCKLPS | Mnemonic::UNPCKHPS => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),

        Mnemonic::CMPPS | Mnemonic::SHUFPS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::MOVHPS | Mnemonic::MOVLPS => ot_chk(
            ins,
            &[
                (&[XMM, M64], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[(M64, M64)],
            &[],
        ),
        Mnemonic::MOVLHPS | Mnemonic::MOVHLPS => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::MOVAPS | Mnemonic::MOVUPS => ot_chk(
            ins,
            &[
                (&[XMM, M128], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
            ],
            &[(M128, M128)],
            &[],
        ),
        Mnemonic::MOVSS => ot_chk(
            ins,
            &[
                (&[XMM, M32], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[(M32, M32)],
            &[],
        ),
        Mnemonic::SQRTSS
        | Mnemonic::ADDSS
        | Mnemonic::SUBSS
        | Mnemonic::DIVSS
        | Mnemonic::MULSS
        | Mnemonic::RCPSS
        | Mnemonic::RSQRTSS
        | Mnemonic::MINSS
        | Mnemonic::MAXSS => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M32], Optional::Needed)],
            &[],
            &[],
        ),

        Mnemonic::ADDPS
        | Mnemonic::SUBPS
        | Mnemonic::DIVPS
        | Mnemonic::MULPS
        | Mnemonic::RCPPS
        | Mnemonic::SQRTPS
        | Mnemonic::RSQRTPS
        | Mnemonic::MINPS
        | Mnemonic::MAXPS
        | Mnemonic::ORPS
        | Mnemonic::ANDPS
        | Mnemonic::XORPS => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),

        // #####  #####  #####   #####
        // #      #      #           #
        // #####  #####  #####   #####
        //     #      #  #       #
        // #####  #####  #####   #####
        // (SSE2)
        Mnemonic::MOVDQ2Q => ot_chk(
            ins,
            &[(&[MMX], Optional::Needed), (&[XMM], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::MOVMSKPD => ot_chk(
            ins,
            &[(&[R32, R64], Optional::Needed), (&[XMM], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::MOVLPD | Mnemonic::MOVHPD | Mnemonic::MOVSD => ot_chk(
            ins,
            &[
                (&[XMM, M64], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[(M64, M64)],
            &[],
        ),
        Mnemonic::MOVAPD | Mnemonic::MOVUPD | Mnemonic::MOVDQA => ot_chk(
            ins,
            &[
                (&[XMM, M128], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
            ],
            &[(M128, M128)],
            &[],
        ),
        Mnemonic::CMPSD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),

        Mnemonic::CMPPD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),

        Mnemonic::SQRTSD
        | Mnemonic::ADDSD
        | Mnemonic::SUBSD
        | Mnemonic::DIVSD
        | Mnemonic::MULSD
        | Mnemonic::MINSD
        | Mnemonic::COMISD
        | Mnemonic::UCOMISD
        | Mnemonic::MAXSD => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M64], Optional::Needed)],
            &[],
            &[],
        ),

        Mnemonic::ADDPD
        | Mnemonic::SUBPD
        | Mnemonic::DIVPD
        | Mnemonic::MULPD
        | Mnemonic::SQRTPD
        | Mnemonic::MINPD
        | Mnemonic::MAXPD
        | Mnemonic::ORPD
        | Mnemonic::ANDPD
        | Mnemonic::XORPD => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::MASKMOVDQU => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::MOVNTDQ | Mnemonic::MOVNTPD => ot_chk(
            ins,
            &[(&[M128], Optional::Needed), (&[XMM], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::MOVNTI => ot_chk(
            ins,
            &[
                (&[M32, M64], Optional::Needed),
                (&[R32, R64], Optional::Needed),
            ],
            &[],
            &[],
        ),

        // #   #  #   #  #   #
        // ## ##  ## ##   # #
        // # # #  # # #    #
        // #   #  #   #   # #
        // #   #  #   #  #   #
        // (MMX/SSE2)
        Mnemonic::EMMS => ot_chk(ins, &[], &[], &[]),
        Mnemonic::MOVD => ot_chk(
            ins,
            &[
                (&[MMX, XMM, R32, M32], Optional::Needed),
                (&[MMX, XMM, R32, M32], Optional::Needed),
            ],
            &[
                (M32, M32),
                (R32, R32),
                (MMX, MMX),
                (XMM, XMM),
                (XMM, MMX),
                (MMX, XMM),
            ],
            &[],
        ),
        Mnemonic::MOVQ => ot_chk(
            ins,
            &[
                (&[MMX, XMM, R64, M64], Optional::Needed),
                (&[MMX, XMM, R64, M64], Optional::Needed),
            ],
            &[
                (M64, M64),
                (R64, R64),
                (MMX, MMX),
                (XMM, XMM),
                (XMM, MMX),
                (MMX, XMM),
            ],
            &[],
        ),
        Mnemonic::PSLLW
        | Mnemonic::PSLLD
        | Mnemonic::PSLLQ
        | Mnemonic::PSRLW
        | Mnemonic::PSRLD
        | Mnemonic::PSRLQ
        | Mnemonic::PSRAD
        | Mnemonic::PSRAW => ot_chk(
            ins,
            &[
                (&[MMX, XMM], Optional::Needed),
                (&[I8, MMX, XMM, M64, M128], Optional::Needed),
            ],
            &[(XMM, MMX), (MMX, XMM), (MMX, M128), (XMM, M64)],
            &[],
        ),
        Mnemonic::PADDB
        | Mnemonic::PADDW
        | Mnemonic::PADDD
        | Mnemonic::PADDQ
        | Mnemonic::PADDSB
        | Mnemonic::PADDSW
        | Mnemonic::PADDUSB
        | Mnemonic::PADDUSW
        | Mnemonic::PSUBB
        | Mnemonic::PSUBW
        | Mnemonic::PSUBSB
        | Mnemonic::PSUBSW
        | Mnemonic::PSUBUSB
        | Mnemonic::PSUBUSW
        | Mnemonic::PMULHW
        | Mnemonic::PMULLW
        | Mnemonic::PMADDWD
        | Mnemonic::PCMPGTB
        | Mnemonic::PCMPGTW
        | Mnemonic::PCMPGTD
        | Mnemonic::PCMPEQB
        | Mnemonic::PCMPEQW
        | Mnemonic::PCMPEQD
        | Mnemonic::PACKSSWB
        | Mnemonic::PACKSSDW
        | Mnemonic::PACKUSWB
        | Mnemonic::PUNPCKLBW
        | Mnemonic::PUNPCKLWD
        | Mnemonic::PUNPCKLDQ
        | Mnemonic::PUNPCKHBW
        | Mnemonic::PUNPCKHWD
        | Mnemonic::PUNPCKHDQ
        | Mnemonic::PAND
        | Mnemonic::PANDN
        | Mnemonic::POR
        | Mnemonic::PXOR
        | Mnemonic::PSUBD => ot_chk(
            ins,
            &[
                (&[MMX, XMM], Optional::Needed),
                (&[MMX, XMM, M64, M128], Optional::Needed),
            ],
            &[(XMM, MMX), (MMX, XMM), (XMM, M64), (MMX, M128)],
            &[],
        ),
        Mnemonic::PMULUDQ | Mnemonic::PSUBQ => ot_chk(
            ins,
            &[
                (&[MMX, XMM], Optional::Needed),
                (&[MMX, M64, XMM, M128], Optional::Needed),
            ],
            &[(MMX, XMM), (XMM, MMX), (XMM, M64), (MMX, M128)],
            &[],
        ),
        Mnemonic::PSHUFLW | Mnemonic::PSHUFHW | Mnemonic::PSHUFD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::PSLLDQ | Mnemonic::PSRLDQ => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[I8], Optional::Needed)],
            &[],
            &[],
        ),

        // #####  #####  #####   ####
        // #      #      #           #
        // #####  #####  #####   ####
        //     #      #  #       #
        // #####  #####  #####   ####
        // (SSE 3)
        Mnemonic::ADDSUBPD
        | Mnemonic::ADDSUBPS
        | Mnemonic::HADDPD
        | Mnemonic::HADDPS
        | Mnemonic::HSUBPS
        | Mnemonic::HSUBPD
        | Mnemonic::MOVSLDUP
        | Mnemonic::MOVSHDUP => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),

        // weird one
        Mnemonic::LDDQU => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::MOVDDUP => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M64], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::MONITOR
        | Mnemonic::MWAIT
        | Mnemonic::MFENCE
        | Mnemonic::LFENCE
        | Mnemonic::CLFLUSH => ot_chk(ins, &[], &[], &[]),

        // ##### ##### #####  #####   ####
        // #     #     #      #           #
        // ##### ##### #####  #####   ####
        //     #     #     #  #           #
        // ##### ##### #####  #####   ####
        // (SSSE 3)
        Mnemonic::PABSW
        | Mnemonic::PABSD
        | Mnemonic::PABSB
        | Mnemonic::PSIGNW
        | Mnemonic::PSIGND
        | Mnemonic::PSIGNB
        | Mnemonic::PHSUBW
        | Mnemonic::PHSUBD
        | Mnemonic::PHADDW
        | Mnemonic::PHADDD
        | Mnemonic::PSHUFB
        | Mnemonic::PHSUBSW
        | Mnemonic::PHADDSW
        | Mnemonic::PMULHRSW
        | Mnemonic::PMADDUBSW => ot_chk(
            ins,
            &[
                (&[MMX, XMM], Optional::Needed),
                (&[MMX, XMM, M64, M128], Optional::Needed),
            ],
            &[(MMX, XMM), (XMM, M64), (XMM, MMX), (MMX, M128)],
            &[],
        ),

        Mnemonic::PALIGNR => ot_chk(
            ins,
            &[
                (&[MMX, XMM], Optional::Needed),
                (&[MMX, XMM, M64, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[(MMX, XMM), (XMM, M64), (XMM, MMX), (MMX, M128)],
            &[],
        ),

        // #####  #####  #####  #   #
        // #      #      #      #   #
        // #####  #####  #####  #####
        //     #      #  #          #
        // #####  #####  #####      #
        // (SSE4)
        Mnemonic::PINSRB => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[R32, M8], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::PINSRQ => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[R64, M64], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::PINSRD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[R32, M32], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::PEXTRB => ot_chk(
            ins,
            &[
                (&[R32, R64, M32, M64], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::PEXTRD => ot_chk(
            ins,
            &[
                (&[R32, M32], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::PEXTRQ => ot_chk(
            ins,
            &[
                (&[R64, M64], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::PEXTRW => ot_chk(
            ins,
            &[
                (&[M16], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::PTEST
        | Mnemonic::PMAXSB
        | Mnemonic::PMAXSD
        | Mnemonic::PMINSD
        | Mnemonic::PMINSB
        | Mnemonic::PMINUW
        | Mnemonic::PMULDQ
        | Mnemonic::PMULLD
        | Mnemonic::PCMPEQQ
        | Mnemonic::PCMPGTQ
        | Mnemonic::BLENDVPS
        | Mnemonic::BLENDVPD
        | Mnemonic::PACKUSDW => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::POPCNT => ot_chk(
            ins,
            &[
                (&[R16, R32, R64], Optional::Needed),
                (&[R16, M16, R32, M32, R64, M64], Optional::Needed),
            ],
            &[
                (R16, M32),
                (R16, M64),
                (R16, R32),
                (R16, R64),
                (R32, R16),
                (R32, R64),
                (R32, M16),
                (R32, M64),
                (R64, R16),
                (R64, R32),
                (R64, M16),
                (R64, M32),
            ],
            &[],
        ),
        Mnemonic::MOVNTDQA => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::EXTRACTPS => ot_chk(
            ins,
            &[
                (&[R32, R64, M32], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::ROUNDSS | Mnemonic::ROUNDSD | Mnemonic::INSERTPS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::DPPS
        | Mnemonic::DPPD
        | Mnemonic::BLENDPS
        | Mnemonic::BLENDPD
        | Mnemonic::PBLENDW
        | Mnemonic::ROUNDPS
        | Mnemonic::ROUNDPD
        | Mnemonic::MPSADBW
        | Mnemonic::PCMPESTRI
        | Mnemonic::PCMPESTRM
        | Mnemonic::PCMPISTRM
        | Mnemonic::PCMPISTRI => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::PMOVZXBW
        | Mnemonic::PMOVZXWD
        | Mnemonic::PMOVZXDQ
        | Mnemonic::PMOVSXBW
        | Mnemonic::PMOVSXWD
        | Mnemonic::PMOVSXDQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM], true)
                .push(&[XMM, M64], true)
                .set_mode(CheckMode::NOSIZE)
                .check(ins)
        }
        Mnemonic::PMOVZXBD | Mnemonic::PMOVZXWQ | Mnemonic::PMOVSXBD | Mnemonic::PMOVSXWQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM], true)
                .push(&[XMM, M32], true)
                .set_mode(CheckMode::NOSIZE)
                .check(ins)
        }
        Mnemonic::PMOVZXBQ | Mnemonic::PMOVSXBQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM], true)
                .push(&[XMM, M16], true)
                .set_mode(CheckMode::NOSIZE)
                .check(ins)
        }
        //  ###   #   #  #   #
        // #   #  #   #   # #
        // #   #   # #     #
        // #####   # #    # #
        // #   #    #    #   #
        // AVX chk

        // idk derived
        Mnemonic::VPMOVSXBW | Mnemonic::VPMOVSXWD | Mnemonic::VPMOVSXDQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, M64, M128, M256], true)
                .set_mode(CheckMode::AVX)
                .forbidden(&[
                    [XMM, YMM],
                    [XMM, M128],
                    [XMM, M256],
                    [YMM, M64],
                    [YMM, M256],
                    [ZMM, XMM],
                    [ZMM, M64],
                    [ZMM, M128],
                ])
                .set_evex()
                .check(ins)
        }
        Mnemonic::VPMOVSXBD | Mnemonic::VPMOVSXWQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, M32, M64, M128], true)
                .set_mode(CheckMode::AVX)
                .forbidden(&[
                    [XMM, YMM],
                    [XMM, M64],
                    [XMM, M128],
                    [YMM, M32],
                    [YMM, M128],
                    [ZMM, XMM],
                    [ZMM, M32],
                    [ZMM, M64],
                ])
                .set_evex()
                .check(ins)
        }
        Mnemonic::VPMOVSXBQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, M16, M32, M64], true)
                .set_mode(CheckMode::AVX)
                .forbidden(&[
                    [XMM, YMM],
                    [XMM, M32],
                    [XMM, M64],
                    [YMM, M16],
                    [YMM, M64],
                    [ZMM, XMM],
                    [ZMM, M16],
                    [ZMM, M32],
                ])
                .set_evex()
                .check(ins)
        }
        Mnemonic::VPINSRB => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[R32, M8], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VPINSRQ => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[R64, M64], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VPINSRD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[R32, M32], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VPEXTRB => ot_chk(
            ins,
            &[
                (&[M8, R32, R64, M32, M64], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VPEXTRD | Mnemonic::VEXTRACTPS => ot_chk(
            ins,
            &[
                (&[R32, M32], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VPEXTRQ => ot_chk(
            ins,
            &[
                (&[R64, M64], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VPEXTRW => ot_chk(
            ins,
            &[
                (&[M16], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VROUNDSS | Mnemonic::VINSERTPS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VROUNDSD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VROUNDPS | Mnemonic::VROUNDPD => ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[(XMM, YMM), (XMM, M256), (YMM, M128), (YMM, XMM)],
            &[],
        ),
        Mnemonic::VMOVAPS
        | Mnemonic::VMOVAPD
        | Mnemonic::VMOVUPS
        | Mnemonic::VMOVUPD
        | Mnemonic::VMOVDQA => ot_chk(
            ins,
            &[
                (&[XMM, YMM, ZMM, M128, M256, M512], Optional::Needed),
                (&[XMM, YMM, ZMM, M128, M256, M512], Optional::Needed),
            ],
            &[
                (XMM, M256),
                (XMM, YMM),
                (YMM, XMM),
                (YMM, M128),
                (MA, MA),
                (XMM, M512),
                (YMM, M512),
                (XMM, ZMM),
                (YMM, ZMM),
                (ZMM, M128),
                (ZMM, XMM),
                (ZMM, YMM),
                (ZMM, M256),
            ],
            &[],
        ),
        Mnemonic::VMOVMSKPD => avx_ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VMOVSD => avx_ot_chk(
            ins,
            &[
                (&[XMM, M64], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
                (&[XMM], Optional::Optional),
            ],
            &[(MA, MA, XMM), (XMM, MA, XMM), (MA, XMM, XMM)],
            &[],
        ),
        Mnemonic::VMOVSS => avx_ot_chk(
            ins,
            &[
                (&[XMM, M32], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
                (&[XMM], Optional::Optional),
            ],
            &[(MA, MA, XMM), (XMM, MA, XMM), (MA, XMM, XMM)],
            &[],
        ),
        Mnemonic::VPMULDQ | Mnemonic::VPMULLD => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
            ],
            &[
                (XMM, YMM, M128),
                (XMM, YMM, M256),
                (YMM, XMM, M128),
                (YMM, XMM, M256),
                (YMM, YMM, M128),
                (XMM, XMM, M256),
            ],
            &[],
        ),
        Mnemonic::VMOVLPS | Mnemonic::VMOVLPD | Mnemonic::VMOVHPS | Mnemonic::VMOVHPD => {
            avx_ot_chk(
                ins,
                &[
                    (&[XMM, M64], Optional::Needed),
                    (&[XMM], Optional::Needed),
                    (&[M64], Optional::Optional),
                ],
                &[(MA, XMM, MA)],
                &[],
            )
        }
        Mnemonic::VLDDQU | Mnemonic::VMOVNTDQA => ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[M128, M256], Optional::Needed),
            ],
            &[(XMM, M256), (YMM, M128)],
            &[],
        ),
        Mnemonic::VPHMINPOSUW => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::VMOVDDUP => ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M64, M256], Optional::Needed),
            ],
            &[(XMM, YMM), (XMM, M256), (YMM, XMM), (YMM, M64)],
            &[],
        ),
        Mnemonic::VMOVSLDUP
        | Mnemonic::VPTEST
        | Mnemonic::VMOVSHDUP
        | Mnemonic::VRCPPS
        | Mnemonic::VSQRTPS
        | Mnemonic::VRSQRTPS
        | Mnemonic::VSQRTPD => ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
            ],
            &[(XMM, M256), (XMM, YMM), (YMM, XMM), (YMM, M128)],
            &[],
        ),
        Mnemonic::VADDSD
        | Mnemonic::VSUBSD
        | Mnemonic::VMULSD
        | Mnemonic::VDIVSD
        | Mnemonic::VSQRTSD
        | Mnemonic::VMINSD
        | Mnemonic::VMAXSD => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VADDSS
        | Mnemonic::VSUBSS
        | Mnemonic::VMULSS
        | Mnemonic::VDIVSS
        | Mnemonic::VRCPSS
        | Mnemonic::VSQRTSS
        | Mnemonic::VRSQRTSS
        | Mnemonic::VMINSS
        | Mnemonic::VMAXSS => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VADDPD
        | Mnemonic::VSUBPD
        | Mnemonic::VDIVPD
        | Mnemonic::VMULPD
        | Mnemonic::VMINPD
        | Mnemonic::VMAXPD
        | Mnemonic::VORPD
        | Mnemonic::VANDNPD
        | Mnemonic::VANDPD
        | Mnemonic::VXORPD
        | Mnemonic::VADDPS
        | Mnemonic::VSUBPS
        | Mnemonic::VDIVPS
        | Mnemonic::VMULPS
        | Mnemonic::VMINPS
        | Mnemonic::VMAXPS
        | Mnemonic::VORPS
        | Mnemonic::VANDNPS
        | Mnemonic::VANDPS
        | Mnemonic::VUNPCKLPS
        | Mnemonic::VUNPCKHPS
        | Mnemonic::VADDSUBPS
        | Mnemonic::VADDSUBPD
        | Mnemonic::VHADDPS
        | Mnemonic::VHADDPD
        | Mnemonic::VHSUBPS
        | Mnemonic::VHSUBPD
        | Mnemonic::VPMAXSB
        | Mnemonic::VPMAXSD
        | Mnemonic::VPMINSB
        | Mnemonic::VPMINSD
        | Mnemonic::VPMAXUW
        | Mnemonic::VPMAXUB
        | Mnemonic::VPMINUW
        | Mnemonic::VPMINUB
        | Mnemonic::VPCMPEQQ
        | Mnemonic::VPCMPGTQ
        | Mnemonic::VPACKUSDW
        | Mnemonic::VXORPS => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, YMM, ZMM], Optional::Needed),
                (
                    &[XMM, YMM, ZMM, M128, M256, M512, MBCST32, MBCST64],
                    Optional::Needed,
                ),
            ],
            &[
                (XMM, YMM, M128),
                (XMM, YMM, M256),
                (YMM, XMM, M128),
                (YMM, XMM, M256),
                (YMM, YMM, M128),
                (XMM, XMM, M256),
            ],
            &[],
        ),
        Mnemonic::VCOMISD | Mnemonic::VUCOMISD => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M64], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::VCOMISS | Mnemonic::VUCOMISS => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M32], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::VMOVHLPS | Mnemonic::VMOVLHPS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VPCMPESTRI
        | Mnemonic::VPCMPESTRM
        | Mnemonic::VPCMPISTRI
        | Mnemonic::VPCMPISTRM => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VCMPSS => ot_chk(
            ins,
            &[
                (&[XMM, K], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),

        Mnemonic::VCMPSD => ot_chk(
            ins,
            &[
                (&[XMM, K], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VBLENDVPS | Mnemonic::VBLENDVPD | Mnemonic::VPBLENDVB => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
            ],
            &[
                (XMM, YMM, M128),
                (XMM, YMM, M256),
                (YMM, XMM, M128),
                (YMM, XMM, M256),
                (YMM, YMM, M128),
                (XMM, XMM, M256),
            ],
            &[],
        ),

        Mnemonic::VBLENDPS
        | Mnemonic::VBLENDPD
        | Mnemonic::VPBLENDW
        | Mnemonic::VMPSADBW
        | Mnemonic::VDPPS
        | Mnemonic::VDPPD
        | Mnemonic::VCMPPS
        | Mnemonic::VCMPPD
        | Mnemonic::VSHUFPS => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, YMM, M128, M256, M512], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[
                (XMM, YMM, M128),
                (XMM, YMM, M256),
                (YMM, XMM, M128),
                (YMM, XMM, M256),
                (YMM, YMM, M128),
                (XMM, XMM, M256),
            ],
            &[],
        ),

        // MMX derived
        Mnemonic::VPOR
        | Mnemonic::VPAND
        | Mnemonic::VPXOR
        | Mnemonic::VPADDB
        | Mnemonic::VPADDW
        | Mnemonic::VPADDD
        | Mnemonic::VPADDQ
        | Mnemonic::VPSUBB
        | Mnemonic::VPSUBD
        | Mnemonic::VPSUBQ
        | Mnemonic::VPSUBW
        | Mnemonic::VPANDN
        | Mnemonic::VPSUBSW
        | Mnemonic::VPSUBSB
        | Mnemonic::VPADDSB
        | Mnemonic::VPADDSW
        | Mnemonic::VPMULLW
        | Mnemonic::VPSUBUSB
        | Mnemonic::VPSUBUSW
        | Mnemonic::VPADDUSB
        | Mnemonic::VPADDUSW
        | Mnemonic::VPMADDWD
        | Mnemonic::VPCMPEQB
        | Mnemonic::VPCMPEQW
        | Mnemonic::VPCMPEQD
        | Mnemonic::VPCMPGTB
        | Mnemonic::VPCMPGTW
        | Mnemonic::VPCMPGTD
        | Mnemonic::VPACKUSWB
        | Mnemonic::VPACKSSWB
        | Mnemonic::VPACKSSDW
        | Mnemonic::VPUNPCKLBW
        | Mnemonic::VPUNPCKHBW
        | Mnemonic::VPUNPCKLWD
        | Mnemonic::VPUNPCKHWD
        | Mnemonic::VPUNPCKLDQ
        | Mnemonic::VPUNPCKHDQ
        | Mnemonic::VPMULHW => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM, ZMM, K], Optional::Needed),
                (&[XMM, YMM, ZMM, K], Optional::Needed),
                (&[XMM, YMM, ZMM, M128, M256, M512], Optional::Needed),
            ],
            &[
                (XMM, YMM, M128),
                (XMM, YMM, M256),
                (YMM, XMM, M128),
                (YMM, XMM, M256),
                (YMM, YMM, M128),
                (XMM, XMM, M256),
            ],
            &[],
        ),
        Mnemonic::VPSLLW
        | Mnemonic::VPSLLD
        | Mnemonic::VPSLLQ
        | Mnemonic::VPSRLW
        | Mnemonic::VPSRLD
        | Mnemonic::VPSRLQ
        | Mnemonic::VPSRAD
        | Mnemonic::VPSRAW => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, YMM, ZMM, M128, M256, M512, I8], Optional::Needed),
            ],
            &[
                (XMM, YMM, M128),
                (XMM, YMM, M256),
                (YMM, XMM, M128),
                (YMM, XMM, M256),
                (YMM, YMM, M128),
                (XMM, XMM, M256),
            ],
            &[],
        ),
        Mnemonic::VMOVD => ot_chk(
            ins,
            &[
                (&[XMM, R32, M32], Optional::Needed),
                (&[XMM, R32, M32], Optional::Needed),
            ],
            &[(M32, M32), (R32, R32), (XMM, XMM)],
            &[],
        ),
        Mnemonic::VMOVQ => ot_chk(
            ins,
            &[
                (&[XMM, R64, M64], Optional::Needed),
                (&[XMM, R64, M64], Optional::Needed),
            ],
            &[(M64, M64), (R64, R64), (XMM, XMM)],
            &[],
        ),
        // part2a
        Mnemonic::VZEROALL | Mnemonic::VZEROUPPER => ot_chk(ins, &[], &[], &[]),
        Mnemonic::PAVGB | Mnemonic::PAVGW => ot_chk(
            ins,
            &[
                (&[XMM, MMX], Optional::Needed),
                (&[XMM, MMX], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VPAVGB
        | Mnemonic::VPAVGW
        | Mnemonic::VPHADDW
        | Mnemonic::VPHADDD
        | Mnemonic::VPHSUBW
        | Mnemonic::VPHSUBD => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, YMM, ZMM, M128, M256, M512], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VBROADCASTF128 => avx_ot_chk_wthout(
            ins,
            &[(&[YMM], Optional::Needed), (&[M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::VBROADCASTSD => avx_ot_chk_wthout(
            ins,
            &[
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VBROADCASTSS => avx_ot_chk_wthout(
            ins,
            &[
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VEXTRACTF128 => avx_ot_chk_wthout(
            ins,
            &[
                (&[XMM, M128], Optional::Needed),
                (&[YMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VINSERTF128 => avx_ot_chk_wthout(
            ins,
            &[
                (&[YMM], Optional::Needed),
                (&[YMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VPALIGNR => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M256, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        // part2b
        Mnemonic::STMXCSR | Mnemonic::VSTMXCSR | Mnemonic::LDMXCSR | Mnemonic::VLDMXCSR => {
            ot_chk(ins, &[(&[M32], Optional::Needed)], &[], &[])
        }
        Mnemonic::VMOVMSKPS => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VPERMILPD | Mnemonic::VPERMILPS => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, YMM, ZMM, M128, M256, M512], Optional::Needed),
                (&[XMM, YMM, ZMM, M256, M128, M512, I8], Optional::Needed),
            ],
            &[(XMM, MA, MA), (YMM, MA, MA), (ZMM, MA, MA)],
            &[],
        ),
        Mnemonic::VPCLMULQDQ => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, YMM, ZMM, M256, M512, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::PCLMULQDQ => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VPERM2F128 | Mnemonic::VPERM2I128 => avx_ot_chk(
            ins,
            &[
                (&[YMM], Optional::Needed),
                (&[YMM], Optional::Needed),
                (&[YMM, M256], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        // part2c
        Mnemonic::VPMAXSW
        | Mnemonic::VPMINSW
        | Mnemonic::VPSIGNB
        | Mnemonic::VPSIGNW
        | Mnemonic::VPSIGND
        | Mnemonic::VPMULUDQ
        | Mnemonic::VPMULHUW
        | Mnemonic::VPMULHRSW => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VPSRLDQ => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, YMM, ZMM, M128, M256, M512], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VPINSRW => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[R32, M16], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        // part2c-ext
        Mnemonic::VPMAXUD => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, M128, YMM, ZMM, M256, M512], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::PMAXSW | Mnemonic::PMINSW | Mnemonic::PMULHUW => ot_chk(
            ins,
            &[
                (&[XMM, MMX], Optional::Needed),
                (&[XMM, MMX], Optional::Needed),
            ],
            &[(XMM, MMX), (MMX, XMM)],
            &[],
        ),
        Mnemonic::PMAXUD => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::PINSRW => ot_chk(
            ins,
            &[
                (&[XMM, MMX], Optional::Needed),
                (&[R32, M16], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        // fma-part1
        Mnemonic::VFMADD132PD
        | Mnemonic::VFMADD213PD
        | Mnemonic::VFMADD231PD
        | Mnemonic::VFMSUB132PD
        | Mnemonic::VFMSUB213PD
        | Mnemonic::VFMSUB231PD
        | Mnemonic::VFMADD132PS
        | Mnemonic::VFMADD213PS
        | Mnemonic::VFMADD231PS
        | Mnemonic::VFMSUB132PS
        | Mnemonic::VFMSUB213PS
        | Mnemonic::VFMSUB231PS => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VFMADD132SS
        | Mnemonic::VFMADD213SS
        | Mnemonic::VFMADD231SS
        | Mnemonic::VFMSUB132SS
        | Mnemonic::VFMSUB213SS
        | Mnemonic::VFMSUB231SS => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VFMADD132SD
        | Mnemonic::VFMADD213SD
        | Mnemonic::VFMADD231SD
        | Mnemonic::VFMSUB132SD
        | Mnemonic::VFMSUB213SD
        | Mnemonic::VFMSUB231SD => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        // fma-part2
        Mnemonic::VFNMADD132PD
        | Mnemonic::VFNMADD213PD
        | Mnemonic::VFNMADD231PD
        | Mnemonic::VFNMSUB132PD
        | Mnemonic::VFNMSUB213PD
        | Mnemonic::VFNMSUB231PD
        | Mnemonic::VFNMADD132PS
        | Mnemonic::VFNMADD213PS
        | Mnemonic::VFNMADD231PS
        | Mnemonic::VFNMSUB132PS
        | Mnemonic::VFNMSUB213PS
        | Mnemonic::VFNMSUB231PS => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VFNMADD132SS
        | Mnemonic::VFNMADD213SS
        | Mnemonic::VFNMADD231SS
        | Mnemonic::VFNMSUB132SS
        | Mnemonic::VFNMSUB213SS
        | Mnemonic::VFNMSUB231SS => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VFNMADD132SD
        | Mnemonic::VFNMADD213SD
        | Mnemonic::VFNMADD231SD
        | Mnemonic::VFNMSUB132SD
        | Mnemonic::VFNMSUB213SD
        | Mnemonic::VFNMSUB231SD => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        // fma-part3
        Mnemonic::VFMADDSUB132PS
        | Mnemonic::VFMADDSUB213PS
        | Mnemonic::VFMADDSUB231PS
        | Mnemonic::VFMSUBADD132PS
        | Mnemonic::VFMSUBADD213PS
        | Mnemonic::VFMSUBADD231PS
        | Mnemonic::VFMADDSUB132PD
        | Mnemonic::VFMADDSUB213PD
        | Mnemonic::VFMADDSUB231PD
        | Mnemonic::VFMSUBADD132PD
        | Mnemonic::VFMSUBADD213PD
        | Mnemonic::VFMSUBADD231PD => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
            ],
            &[],
            &[],
        ),
        // aes
        Mnemonic::AESDEC
        | Mnemonic::AESENC
        | Mnemonic::AESIMC
        | Mnemonic::VAESIMC
        | Mnemonic::AESDECLAST => avx_ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::VAESDEC | Mnemonic::VAESENC | Mnemonic::VAESDECLAST => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, YMM, ZMM], Optional::Needed),
                (&[XMM, YMM, M128, M256, M512], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::AESKEYGENASSIST | Mnemonic::VAESKEYGENASSIST => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        // cvt-part1
        Mnemonic::CVTPD2PI => ot_chk(
            ins,
            &[(&[MMX], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::CVTSS2SI => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::CVTTPD2PI => ot_chk(
            ins,
            &[(&[MMX], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::CVTPS2PI | Mnemonic::CVTTPS2PI => ot_chk(
            ins,
            &[(&[MMX], Optional::Needed), (&[XMM, M64], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::CVTPI2PS | Mnemonic::CVTPI2PD => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[MMX, M64], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::CVTSI2SS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[R32, R64, M32, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::CVTPD2DQ
        | Mnemonic::CVTPS2DQ
        | Mnemonic::CVTTPS2DQ
        | Mnemonic::CVTTPD2DQ
        | Mnemonic::CVTDQ2PS
        | Mnemonic::CVTPD2PS => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::CVTSS2SD => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M32], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::CVTDQ2PD | Mnemonic::CVTPS2PD | Mnemonic::CVTSD2SS => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M64], Optional::Needed)],
            &[],
            &[],
        ),
        Mnemonic::CVTTSS2SI => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::CVTSD2SI | Mnemonic::CVTTSD2SI => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::CVTSI2SD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, R32, R64, M32, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        // cvt-part2
        Mnemonic::VCVTSI2SD | Mnemonic::VCVTSI2SS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[R64, R32, M32, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VCVTSS2SD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VCVTDQ2PS | Mnemonic::VCVTTPD2DQ | Mnemonic::VCVTTPS2DQ => avx_ot_chk(
            ins,
            &[
                (&[YMM, XMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VCVTDQ2PD => ot_chk(
            ins,
            &[
                (&[YMM, XMM], Optional::Needed),
                (&[XMM, M64, M128], Optional::Needed),
            ],
            &[(YMM, M64), (XMM, M128)],
            &[],
        ),
        Mnemonic::VCVTSD2SS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VCVTTSD2SI => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VCVTSS2SI => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VCVTSD2SI => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnemonic::VCVTPD2DQ | Mnemonic::VCVTPD2PS | Mnemonic::VCVTPS2DQ | Mnemonic::VCVTPS2PD => {
            ot_chk(
                ins,
                &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
                &[],
                &[],
            )
        }

        //  ###   #   #  #   #    #  #####
        // #   #  #   #   # #     #  #   #
        // #####   # #     #      #  #   #
        // #   #   # #    # #     #  #   #
        // #   #    #    #   #    #  #####
        //
        // (AVX-10 + AVX-512)
        VADDPH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VADDSH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M16], true)
                .avx10_modifier(AVX10Modifier::ER, Size::Xword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VBLENDMPD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VBLENDMPS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VALIGND => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VALIGNQ => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VBROADCASTF32X2 => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[YMM, ZMM], true)
                .push(&[XMM, M64], true)
                .set_evex()
                .set_mode(CheckMode::AVX)
                .allow_masks()
                .check(ins)
        }
        VBROADCASTF32X4 | VBROADCASTF64X2 => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[YMM, ZMM], true)
                .push(&[XMM, M128], true)
                .set_evex()
                .set_mode(CheckMode::AVX)
                .allow_masks()
                .check(ins)
        }
        VBROADCASTF32X8 | VBROADCASTF64X4 => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[ZMM], true)
                .push(&[M256], true)
                .set_evex()
                .set_mode(CheckMode::AVX)
                .allow_masks()
                .check(ins)
        }
        VBCSTNEBF162PS | VBCSTNESH2PS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM], true)
                .push(&[M16], true)
                .check(ins)
        }
        VCOMISH => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM], true)
                .push(&[XMM, M16], true)
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .set_evex()
                .check(ins)
        }
        VCOMPRESSPD | VCOMPRESSPS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM, M128, M256, M512], true)
                .push(&[XMM, YMM, ZMM], true)
                .allow_masks()
                .set_evex()
                .check(ins)
        }
        VCMPSH => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[K], true)
                .push(&[XMM], true)
                .push(&[XMM, M128, M16], true)
                .push(&[I8], true)
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .set_evex()
                .check(ins)
        }
        VCMPPH => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[K], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .push(&[I8], true)
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .set_evex()
                .check(ins)
        }
        KADDB | KADDW | KADDD | KADDQ | KANDB | KANDW | KANDD | KANDQ | KANDNB | KANDNW
        | KANDND | KANDNQ | KORB | KORW | KORD | KORQ | KXORB | KXORW | KXORD | KXORQ | KXNORB
        | KXNORW | KXNORD | KXNORQ | KUNPCKBW | KUNPCKWD | KUNPCKDQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[K], true)
                .push(&[K], true)
                .push(&[K], true)
                .check(ins)
        }
        KTESTB | KTESTW | KNOTB | KNOTW | KNOTD | KNOTQ | KTESTD | KTESTQ | KORTESTB | KORTESTW
        | KORTESTD | KORTESTQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[K], true)
                .push(&[K], true)
                .check(ins)
        }
        KSHIFTLB | KSHIFTLW | KSHIFTLD | KSHIFTLQ | KSHIFTRB | KSHIFTRW | KSHIFTRD | KSHIFTRQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[K], true)
                .push(&[K], true)
                .push(&[I8], true)
                .check(ins)
        }
        KMOVB | AKMOVB => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[K, R32, M8], true)
                .push(&[K, R32, M8], true)
                .forbidden(&[[R32, M8], [M8, M8], [M8, R32], [R32, R32]])
                .check(ins)
        }
        KMOVW | AKMOVW => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[K, R32, M16], true)
                .push(&[K, R32, M16], true)
                .forbidden(&[[R32, M16], [M16, M16], [M16, R32], [R32, R32]])
                .check(ins)
        }
        KMOVD | AKMOVD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[K, R32, M32], true)
                .push(&[K, R32, M32], true)
                .forbidden(&[[R32, M32], [M32, M32], [M32, R32], [R32, R32]])
                .check(ins)
        }
        KMOVQ | AKMOVQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[K, R64, M64], true)
                .push(&[K, R64, M64], true)
                .forbidden(&[[R64, M64], [M64, R64], [M64, M64], [R64, R64]])
                .check(ins)
        }
        VCVTDQ2PH => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .forbidden(&[
                    [XMM, ZMM],
                    [XMM, M512],
                    [YMM, M128],
                    [YMM, XMM],
                    [YMM, YMM],
                    [YMM, M256],
                ])
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .check(ins)
        }
        VCVTNEPS2BF16 => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .forbidden(&[
                    [XMM, ZMM],
                    [XMM, M512],
                    [YMM, M128],
                    [YMM, XMM],
                    [YMM, YMM],
                    [YMM, M256],
                ])
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTNE2PS2BF16 => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTNEEBF162PS | VCVTNEEPH2PS | VCVTNEOBF162PS | VCVTNEOPH2PS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM], true)
                .push(&[M128, M256], true)
                .check(ins)
        }
        VCVTPD2PH | VCVTPD2UQQ | VCVTPD2QQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTPS2UQQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, M64, M128, M256, MBCST32], true)
                .forbidden(&[
                    [XMM, M256],
                    [XMM, M128],
                    [XMM, YMM],
                    [YMM, YMM],
                    [YMM, M256],
                    [ZMM, XMM],
                    [ZMM, M128],
                    [ZMM, M128],
                ])
                .set_mode(CheckMode::AVX)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTQQ2PH => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTQQ2PD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTPS2UDQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .check(ins)
        }
        VCVTPH2PD | VCVTPH2PSX => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTPH2DQ | VCVTPH2UW | VCVTPH2W => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTQQ2PS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .forbidden(&[
                    [XMM, ZMM],
                    [XMM, M512],
                    [YMM, M128],
                    [YMM, XMM],
                    [YMM, YMM],
                    [YMM, M256],
                ])
                .set_mode(CheckMode::AVX)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTPD2UDQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .forbidden(&[
                    [XMM, ZMM],
                    [XMM, M512],
                    [YMM, M128],
                    [YMM, XMM],
                    [YMM, YMM],
                    [YMM, M256],
                ])
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTPH2PS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, M64, M128, M256], true)
                .forbidden(&[
                    [YMM, M64],
                    [YMM, M256],
                    [XMM, M128],
                    [ZMM, M64],
                    [ZMM, M128],
                    [ZMM, XMM],
                    [XMM, ZMM],
                    [XMM, YMM],
                ])
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTPH2QQ | VCVTPH2UQQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, M32, M64, M128, MBCST16], true)
                .forbidden(&[
                    [XMM, M64],
                    [XMM, M128],
                    [YMM, M32],
                    [YMM, M128],
                    [ZMM, M32],
                    [ZMM, M64],
                ])
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTPH2UDQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, M64, M128, M256, MBCST16], true)
                .forbidden(&[
                    [XMM, M128],
                    [XMM, M256],
                    [YMM, M64],
                    [YMM, M256],
                    [ZMM, M64],
                    [ZMM, M128],
                ])
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTPS2PH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, M64, M128, M256], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[I8], true)
                .forbidden(&[
                    [M64, YMM, I8],
                    [M256, YMM, I8],
                    [M128, XMM, I8],
                    [M64, ZMM, I8],
                    [M128, ZMM, I8],
                    [XMM, ZMM, I8],
                    [YMM, XMM, I8],
                ])
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTPS2PHX => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .forbidden(&[
                    [XMM, ZMM],
                    [XMM, M512],
                    [YMM, XMM],
                    [YMM, YMM],
                    [YMM, M128],
                    [YMM, M256],
                ])
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTPS2QQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, M64, M128, M256, MBCST32], true)
                .forbidden(&[
                    [XMM, YMM],
                    [XMM, M128],
                    [XMM, M256],
                    [YMM, M256],
                    [YMM, M64],
                    [ZMM, M128],
                    [ZMM, XMM],
                    [ZMM, YMM],
                    [ZMM, M256],
                ])
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTUSI2SD | VCVTUSI2SS | VCVTUSI2SH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, R64, R32, M32, M64], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Xword)
                .allow_masks()
                .check(ins)
        }
        VCVTSD2SH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M64], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Xword)
                .allow_masks()
                .check(ins)
        }
        VCVTSH2SI | VCVTSH2USI => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R32, R64], true)
                .push(&[XMM, M16], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Any)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTSS2USI => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R32, R64], true)
                .push(&[XMM, M32], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Any)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTSD2USI => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R32, R64], true)
                .push(&[XMM, M64], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Any)
                .allow_masks()
                .check(ins)
        }
        VCVTSS2SH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M32], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Any)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTSH2SD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M16], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Xword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTSH2SS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M16], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTSI2SH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[R32, R64, M32, M64], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Any)
                .set_evex()
                .check(ins)
        }
        VCVTTSH2SI | VCVTTSH2USI => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R32, R64], true)
                .push(&[XMM, M16], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::SAE, Size::Any)
                .set_evex()
                .check(ins)
        }
        VCVTTSS2SI | VCVTTSS2USI => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R32, R64], true)
                .push(&[XMM, M32], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::SAE, Size::Any)
                .set_evex()
                .check(ins)
        }
        VCVTTSD2USI => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[R32, R64], true)
                .push(&[XMM, M64], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::SAE, Size::Any)
                .set_evex()
                .check(ins)
        }
        VCVTTPD2QQ | VCVTTPD2UQQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTTPS2UQQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, M64, M128, M256, MBCST32], true)
                .set_mode(CheckMode::AVX)
                .forbidden(&[
                    [XMM, M128],
                    [XMM, M256],
                    [XMM, YMM],
                    [ZMM, XMM],
                    [ZMM, M128],
                    [YMM, M64],
                ])
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTUDQ2PD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, M64, M128, M256, MBCST32], true)
                .set_mode(CheckMode::AVX)
                .forbidden(&[
                    [XMM, M128],
                    [XMM, M256],
                    [XMM, YMM],
                    [ZMM, XMM],
                    [ZMM, M128],
                    [YMM, M64],
                ])
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTUQQ2PD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .check(ins)
        }
        VCVTTPS2UDQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTUDQ2PS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTUDQ2PH => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_mode(CheckMode::AVX)
                .forbidden(&[
                    [XMM, M512],
                    [XMM, ZMM],
                    [YMM, XMM],
                    [YMM, M128],
                    [YMM, M256],
                    [YMM, YMM],
                ])
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTUW2PH | VCVTW2PH => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTTPH2DQ | VCVTTPH2UW | VCVTTPH2W => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTUQQ2PH => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VCVTTPH2UQQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, M32, M64, M128, MBCST16], true)
                .set_mode(CheckMode::AVX)
                .forbidden(&[
                    [YMM, M64],
                    [YMM, M128],
                    [ZMM, M32],
                    [ZMM, M64],
                    [XMM, M64],
                    [XMM, M128],
                ])
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTTPH2QQ | VCVTTPH2UDQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, M64, M128, M256, MBCST16], true)
                .set_mode(CheckMode::AVX)
                .forbidden(&[
                    [YMM, M128],
                    [YMM, M256],
                    [ZMM, M64],
                    [ZMM, M128],
                    [XMM, M128],
                    [XMM, M256],
                ])
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTTPS2QQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, M64, M128, M256, MBCST32], true)
                .set_mode(CheckMode::AVX)
                .forbidden(&[
                    [YMM, M128],
                    [YMM, M256],
                    [ZMM, M64],
                    [ZMM, M128],
                    [XMM, M128],
                    [XMM, M256],
                ])
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTTPD2UDQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_mode(CheckMode::AVX)
                .forbidden(&[
                    [XMM, ZMM],
                    [XMM, M512],
                    [YMM, XMM],
                    [YMM, YMM],
                    [YMM, M128],
                    [YMM, M256],
                ])
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VCVTUQQ2PS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_mode(CheckMode::AVX)
                .forbidden(&[
                    [XMM, ZMM],
                    [XMM, M512],
                    [YMM, XMM],
                    [YMM, YMM],
                    [YMM, M128],
                    [YMM, M256],
                ])
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VEXPANDPD | VEXPANDPS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VDBPSADBW => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VDIVPH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VDIVSH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M16], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Xword)
                .allow_masks()
                .check(ins)
        }
        VFCMADDCSH | VFMADDCSH | VFCMULCSH | VFMULCSH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M32], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Xword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VFIXUPIMMSS => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M32], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .allow_masks()
                .check(ins)
        }
        VFIXUPIMMSD => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M64], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VFIXUPIMMPS => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .check(ins)
        }
        VFIXUPIMMPD => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .check(ins)
        }
        VDPBF16PS | VFCMADDCPH | VFMADDCPH | VFCMULCPH | VFMULCPH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .check(ins)
        }
        VEXTRACTF32X4 | VEXTRACTF64X2 | VEXTRACTI32X4 | VEXTRACTI64X2 | VEXTRACTI128 => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, M128], true)
                .push(&[YMM, ZMM], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VEXTRACTF32X8 | VEXTRACTF64X4 | VEXTRACTI32X8 | VEXTRACTI64X4 => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[YMM, M256], true)
                .push(&[ZMM], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VFMADD132PH | VFMADD213PH | VFMADD231PH | VFNMADD132PH | VFNMADD213PH | VFNMADD231PH
        | VFMSUB132PH | VFMSUB213PH | VFMSUB231PH | VFNMSUB132PH | VFNMSUB213PH | VFNMSUB231PH
        | VFMADDSUB132PH | VFMADDSUB213PH | VFMADDSUB231PH | VFMSUBADD132PH | VFMSUBADD213PH
        | VFMSUBADD231PH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VFMADD132SH | VFMADD213SH | VFMADD231SH | VFNMADD132SH | VFNMADD213SH | VFNMADD231SH
        | VFMSUB132SH | VFMSUB213SH | VFMSUB231SH | VFNMSUB132SH | VFNMSUB213SH | VFNMSUB231SH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M16], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VFPCLASSPH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[K], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VFPCLASSPS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[K], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VFPCLASSPD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[K], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VFPCLASSSH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[K], true)
                .push(&[XMM, M16], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VFPCLASSSS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[K], true)
                .push(&[XMM, M32], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VFPCLASSSD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[K], true)
                .push(&[XMM, M64], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VGETEXPPS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .check(ins)
        }
        VGETEXPPH => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .check(ins)
        }
        VGETEXPPD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .check(ins)
        }
        VGETEXPSS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M32], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .allow_masks()
                .check(ins)
        }
        VGETEXPSH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M16], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .allow_masks()
                .check(ins)
        }
        VGETMANTPH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .push(&[I8], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VGETMANTPS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .push(&[I8], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VGETMANTPD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .push(&[I8], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VGETMANTSH => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M16], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VGETMANTSS => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M32], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VGETMANTSD => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M64], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VINSERTI128 => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[YMM], true)
                .push(&[YMM], true)
                .push(&[XMM, M128], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VINSERTF32X4 | VINSERTF64X2 | VINSERTI32X4 | VINSERTI64X2 => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[YMM, ZMM], true)
                .push(&[YMM, ZMM], true)
                .push(&[XMM, M128], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VINSERTF32X8 | VINSERTF64X4 | VINSERTI32X8 | VINSERTI64X4 => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[ZMM], true)
                .push(&[ZMM], true)
                .push(&[XMM, M128], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VMASKMOVPS | VMASKMOVPD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, M128, M256], true)
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM, M128, M256], true)
                .forbidden(&[[MA, RA, MA]])
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VMULPH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }

        VMAXPH | VMINPH | VSUBPH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VMULSH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M16], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Xword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VMAXSH | VMINSH | VSUBSH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M16], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VMOVSH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, M16], true)
                .push(&[XMM, M16], true)
                .push(&[XMM], false)
                .forbidden(&[[MA, MA, XMM], [MA, XMM, XMM], [XMM, MA, XMM]])
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VMOVW => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, R16, M16], true)
                .push(&[XMM, R16, M16], true)
                .forbidden(&[[MA, MA]])
                .set_mode(CheckMode::AVX)
                .check(ins)
        }
        VP2INTERSECTQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[K], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VP2INTERSECTD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[K], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPBLENDD => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM, M128, M256], true)
                .push(&[I8], true)
                .check(ins)
        }
        VPBLENDMB | VPBLENDMW => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPBLENDMD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPBLENDMQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPBROADCASTB => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, M8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPBROADCASTW => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, M16], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPBROADCASTD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, M32], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPBROADCASTQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, M64], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPBROADCASTI32X2 => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, M64], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPBROADCASTI32X8 | VPBROADCASTI64X4 => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[ZMM], true)
                .push(&[M256], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPBROADCASTI32X4 | VPBROADCASTI64X2 => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[YMM, ZMM], true)
                .push(&[M128], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPBROADCASTI128 => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[YMM], true)
                .push(&[M128], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPBROADCASTMB2Q | VPBROADCASTMW2D => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[K], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .check(ins)
        }
        VPCMPB | VPCMPUB | VPCMPW | VPCMPUW => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[K], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPCMPQ | VPCMPUQ => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[K], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPCMPD | VPCMPUD => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[K], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPCOMPRESSB | VPCOMPRESSW => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM, M128, M256, M512], true)
                .push(&[XMM, YMM, ZMM], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPCOMPRESSD | VPCOMPRESSQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM, M128, M256, M512], true)
                .push(&[XMM, YMM, ZMM], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPCONFLICTD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPCONFLICTQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPDPBSSD | VPDPBSSDS | VPDPBSUD | VPDPBSUDS | VPDPBUUD | VPDPBUUDS | VPDPWSUD
        | VPDPWSUDS | VPDPWUSD | VPDPWUUD | VPDPWUUDS | VPDPWUSDS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM, M128, M256], true)
                .check(ins)
        }
        VPERMI2Q | VPERMI2PD | VPERMT2Q | VPERMT2PD | VPMADD52HUQ | VPMADD52LUQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPDPBUSD | VPDPBUSDS | VPDPWSSD | VPDPWSSDS | VPERMD | VPERMI2D | VPERMI2PS | VPERMT2D
        | VPERMT2PS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPERMB | VPERMW | VPERMI2B | VPERMI2W | VPERMT2B | VPERMT2W => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPERMPS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[YMM, ZMM], true)
                .push(&[YMM, ZMM], true)
                .push(&[YMM, ZMM, M256, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPERMPD | VPERMQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[YMM, ZMM], true)
                .push(&[YMM, ZMM, M256, M512, MBCST64], true)
                .push(&[YMM, ZMM, M256, M512, MBCST64, I8], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPEXPANDB | VPEXPANDW | VPEXPANDD | VPEXPANDQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPLZCNTD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPLZCNTQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPMASKMOVD | VPMASKMOVQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, M128, M256], true)
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM, M128, M256], true)
                .forbidden(&[[MA, RA, MA]])
                .check(ins)
        }
        VPMOVB2M | VPMOVW2M | VPMOVD2M | VPMOVQ2M => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[K], true)
                .push(&[XMM, YMM, ZMM], true)
                .set_evex()
                .check(ins)
        }
        VPMOVDB | VPMOVSDB | VPMOVUSDB | VPMOVQW | VPMOVSQW | VPMOVUSQW => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, M32, M64, M128], true)
                .push(&[XMM, YMM, ZMM], true)
                .set_mode(CheckMode::AVX)
                .forbidden(&[
                    [M32, YMM],
                    [M32, ZMM],
                    [M64, XMM],
                    [M64, ZMM],
                    [M128, XMM],
                    [M128, YMM],
                ])
                .set_evex()
                .check(ins)
        }
        VPMOVDW | VPMOVSDW | VPMOVUSDW | VPMOVQD | VPMOVSQD | VPMOVUSQD | VPMOVWB | VPMOVSWB
        | VPMOVUSWB => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, M64, M128, M256], true)
                .push(&[XMM, YMM, ZMM], true)
                .set_mode(CheckMode::AVX)
                .forbidden(&[
                    [M64, YMM],
                    [M64, ZMM],
                    [M128, XMM],
                    [M128, ZMM],
                    [M256, XMM],
                    [M256, YMM],
                ])
                .set_evex()
                .check(ins)
        }
        VPMOVM2B | VPMOVM2W | VPMOVM2D | VPMOVM2Q => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[K], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .check(ins)
        }
        VPMOVQB | VPMOVSQB | VPMOVUSQB => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, M16, M32, M64], true)
                .push(&[XMM, YMM, ZMM], true)
                .set_mode(CheckMode::AVX)
                .forbidden(&[
                    [M16, YMM],
                    [M16, ZMM],
                    [M32, XMM],
                    [M32, ZMM],
                    [M64, XMM],
                    [M64, YMM],
                ])
                .set_evex()
                .check(ins)
        }
        VPOPCNTD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPMULTISHIFTQB | VPOPCNTQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPOPCNTB | VPOPCNTW | VPSHLDVW => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPROLVQ | VPRORVQ | VPSHLDVD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPROLVD | VPRORVD | VPSHLDVQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPROLQ | VPRORQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPROLD | VPRORD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }

        VPSHLDW | VPSHRDW => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPSHLDD | VPSHRDD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPSHLDQ | VPSHRDQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPSHUFBITQMB => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[K], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPSLLVW | VPSRAVW | VPSRLVW => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPSLLVQ | VPSRAVQ | VPSRLVQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPSLLVD | VPSRLVD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }

        VPTERNLOGQ => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPTERNLOGD => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPTESTMB | VPTESTMW | VPTESTNMB | VPTESTNMW => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[K], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPTESTMD | VPTESTNMD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[K], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPTESTMQ | VPTESTNMQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[K], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VRANGESS => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M32], true)
                .push(&[I8], true)
                .set_evex()
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .allow_masks()
                .check(ins)
        }
        VRANGESD => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M64], true)
                .push(&[I8], true)
                .set_evex()
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .allow_masks()
                .check(ins)
        }
        VRANGEPS => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .push(&[I8], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VRANGEPD => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .check(ins)
        }

        VRCP14SS | VRSQRT14SS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M32], true)
                .set_evex()
                .set_mode(CheckMode::AVX)
                .allow_masks()
                .check(ins)
        }
        VSCALEFSS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M32], true)
                .set_evex()
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Xword)
                .allow_masks()
                .check(ins)
        }
        VSCALEFSD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M64], true)
                .set_evex()
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Xword)
                .allow_masks()
                .check(ins)
        }
        VRCP14SD | VRSQRT14SD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M64], true)
                .set_evex()
                .set_mode(CheckMode::AVX)
                .allow_masks()
                .check(ins)
        }
        VRCPPH | VSCALEFPH | VRSQRTPH | VSQRTPH => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VRCP14PS | VRSQRT14PS | VSCALEFPS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VRCP14PD | VRSQRT14PD | VSCALEFPD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::ER, Size::Zword)
                .allow_masks()
                .check(ins)
        }
        VRCPSH | VRSQRTSH | VSQRTSH | VSCALEFSH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M16], true)
                .set_evex()
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::ER, Size::Xword)
                .allow_masks()
                .check(ins)
        }

        VREDUCEPH | VRNDSCALEPH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .check(ins)
        }
        VREDUCEPS | VRNDSCALEPS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .check(ins)
        }
        VREDUCEPD | VRNDSCALEPD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .check(ins)
        }
        VREDUCESH | VRNDSCALESH => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M16], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VREDUCESS | VRNDSCALESS => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M32], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VREDUCESD | VRNDSCALESD => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M64], true)
                .push(&[I8], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }

        VSHA512MSG1 | VSHA512MSG2 => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[YMM], true)
                .push(&[YMM], true)
                .check(ins)
        }
        VSM3RNDS2 => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M128], true)
                .push(&[XMM], true)
                .check(ins)
        }
        VSM4KEY4 | VSM4RNDS4 | VTESTPD | VTESTPS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM], true)
                .push(&[XMM, YMM, M128, M256], true)
                .check(ins)
        }
        VUCOMISH => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[XMM], true)
                .push(&[XMM, M16], true)
                .set_mode(CheckMode::AVX)
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .check(ins)
        }
        VSM3MSG1 | VSM3MSG2 => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M128], true)
                .check(ins)
        }
        VSHA512RNDS2 => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[YMM], true)
                .push(&[YMM], true)
                .push(&[XMM], true)
                .set_mode(CheckMode::AVX)
                .check(ins)
        }
        VSHUFI64X2 | VSHUFF64X2 => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[YMM, ZMM], true)
                .push(&[YMM, ZMM], true)
                .push(&[YMM, ZMM, M256, M512, MBCST64], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VSHUFF32X4 | VSHUFI32X4 => {
            use chkn::*;
            CheckAPI::<4>::new()
                .push(&[YMM, ZMM], true)
                .push(&[YMM, ZMM], true)
                .push(&[YMM, ZMM, M256, M512, MBCST32], true)
                .push(&[I8], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        PREFETCHWT1 => {
            use chkn::*;
            CheckAPI::<1>::new().push(&[M8], true).check(ins)
        }
        V4FMADDSS | V4FNMADDSS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M128], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        V4FMADDPS | V4FNMADDPS | VP4DPWSSDS | VP4DPWSSD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[ZMM], true)
                .push(&[ZMM], true)
                .push(&[ZMM, M128], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VEXP2PS | VRCP28PS | VRSQRT28PS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[ZMM], true)
                .push(&[ZMM, M512, MBCST32], true)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .check(ins)
        }

        VRCP28SS | VRSQRT28SS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M32], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .check(ins)
        }
        VRCP28SD | VRSQRT28SD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M64], true)
                .set_mode(CheckMode::AVX)
                .set_evex()
                .allow_masks()
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .check(ins)
        }
        VEXP2PD | VRCP28PD | VRSQRT28PD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[ZMM], true)
                .push(&[ZMM, M512, MBCST64], true)
                .avx10_modifier(AVX10Modifier::SAE, Size::Zword)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPGATHERDD | VPGATHERDQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[VM32X, VM32Y, VM32Z], true)
                .push(&[XMM, YMM, ZMM], false)
                .forbidden(&[
                    [XMM, VM32Y, ANY],
                    [XMM, VM32Z, ANY],
                    [YMM, VM32X, ANY],
                    [YMM, VM32Z, ANY],
                    [ZMM, VM32X, ANY],
                    [ZMM, VM32Y, ANY],
                ])
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPGATHERQD | VPGATHERQQ => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[VM64X, VM64Y, VM64Z], true)
                .push(&[XMM, YMM, ZMM], false)
                .forbidden(&[
                    [XMM, VM64Y, ANY],
                    [XMM, VM64Z, ANY],
                    [YMM, VM64X, ANY],
                    [YMM, VM64Z, ANY],
                    [ZMM, VM64X, ANY],
                    [ZMM, VM64Y, ANY],
                ])
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VSCATTERDPS | VSCATTERDPD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[VM32X, VM32Y, VM32Z], true)
                .push(&[XMM, YMM, ZMM], true)
                .forbidden(&[
                    [VM32Y, XMM],
                    [VM32Z, XMM],
                    [VM32X, YMM],
                    [VM32Z, YMM],
                    [VM32X, ZMM],
                    [VM32Y, ZMM],
                ])
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VSCATTERQPS | VSCATTERQPD => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[VM64X, VM64Y, VM64Z], true)
                .push(&[XMM, YMM, ZMM], true)
                .forbidden(&[
                    [VM64Y, XMM],
                    [VM64Z, XMM],
                    [VM64X, YMM],
                    [VM64Z, YMM],
                    [VM64X, ZMM],
                    [VM64Y, ZMM],
                ])
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VGATHERPF0QPS | VGATHERPF1QPS | VSCATTERPF0QPS | VSCATTERPF1QPS => {
            use chkn::*;
            CheckAPI::<1>::new()
                .push(&[VM64Z], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VGATHERPF0QPD | VGATHERPF1QPD | VSCATTERPF0QPD | VSCATTERPF1QPD => {
            use chkn::*;
            CheckAPI::<1>::new()
                .push(&[VM64Z], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VGATHERPF0DPD | VGATHERPF1DPD | VSCATTERPF0DPD | VSCATTERPF1DPD => {
            use chkn::*;
            CheckAPI::<1>::new()
                .push(&[VM32Y], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VGATHERPF0DPS | VGATHERPF1DPS | VSCATTERPF0DPS | VSCATTERPF1DPS => {
            use chkn::*;
            CheckAPI::<1>::new()
                .push(&[VM32Z], true)
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VGETEXPSD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM], true)
                .push(&[XMM], true)
                .push(&[XMM, M64], true)
                .set_evex()
                .avx10_modifier(AVX10Modifier::SAE, Size::Xword)
                .allow_masks()
                .check(ins)
        }
        VGATHERQPS | VGATHERQPD | VGATHERDPS | VGATHERDPD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .push(&[XMM, YMM, ZMM], true)
                .push(&[VM64X, VM64Y, VM64Z], true)
                .push(&[XMM, YMM], false)
                .forbidden(&[
                    [XMM, VM64Y, ANY],
                    [XMM, VM64Z, ANY],
                    [YMM, VM64X, ANY],
                    [YMM, VM64Z, ANY],
                    [ZMM, VM64X, ANY],
                    [ZMM, VM64Y, ANY],
                ])
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPSCATTERQD | VPSCATTERQQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[VM64X, VM64Y, VM64Z], true)
                .push(&[XMM, YMM, ZMM], true)
                .forbidden(&[
                    [VM64Y, XMM],
                    [VM64Z, XMM],
                    [VM64X, YMM],
                    [VM64Z, YMM],
                    [VM64X, ZMM],
                    [VM64Y, ZMM],
                ])
                .set_evex()
                .allow_masks()
                .check(ins)
        }
        VPSCATTERDD | VPSCATTERDQ => {
            use chkn::*;
            CheckAPI::<2>::new()
                .push(&[VM32X, VM32Y, VM32Z], true)
                .push(&[XMM, YMM, ZMM], true)
                .forbidden(&[
                    [VM32Y, XMM],
                    [VM32Z, XMM],
                    [VM32X, YMM],
                    [VM32Z, YMM],
                    [VM32X, ZMM],
                    [VM32Y, ZMM],
                ])
                .set_evex()
                .allow_masks()
                .check(ins)
        }

        _ => {
            let er = Error::new(
                "internal error: instruction does not have entry in check layer",
                500,
            );
            Err(er)
        }
    }
}

// Utils

// Legacy check API
#[derive(PartialEq, Debug)]
enum Optional {
    Needed,
    Optional,
}

fn avx_ot_chk_wthout(
    ins: &Instruction,
    ops: &[(&[AType], Optional)],
    forb: &[(AType, AType, AType)],
    addt: &[Mnemonic],
) -> Result<(), Error> {
    if let Some(err) = addt_chk(ins, addt) {
        return Err(err);
    }
    if ops.is_empty() && !ins.is_empty() {
        let er = Error::new(
            "this mnemonic does not accept any operand, but you tried to use one",
            9,
        );
        return Err(er);
    }
    for (idx, allowed) in ops.iter().enumerate() {
        if let Some(op) = ins.get(idx) {
            if let Some(err) = type_check(&op, allowed.0, idx) {
                return Err(err);
            }
        } else if allowed.1 == Optional::Needed {
            let er = Error::new(
                format!("this mnemonic requires operand at index {idx}, but one was not found"),
                9,
            );
            return Err(er);
        } else {
            break;
        }
    }
    if ops.len() == 2 {
        if let Some(err) = size_chk(ins) {
            return Err(err);
        }
    }
    if let Some(err) = avx_forb_chk(ins, forb) {
        return Err(err);
    }
    Ok(())
}
fn avx_ot_chk(
    ins: &Instruction,
    ops: &[(&[AType], Optional)],
    forb: &[(AType, AType, AType)],
    addt: &[Mnemonic],
) -> Result<(), Error> {
    if let Some(err) = addt_chk(ins, addt) {
        return Err(err);
    }
    if ops.is_empty() && !ins.is_empty() {
        let er = Error::new(
            "this mnemonic does not accept any operand, but you tried to use one",
            9,
        );
        return Err(er);
    }
    for (idx, allowed) in ops.iter().enumerate() {
        if let Some(op) = ins.get(idx) {
            if let Some(err) = type_check(&op, allowed.0, idx) {
                return Err(err);
            }
        } else if allowed.1 == Optional::Needed {
            let er = Error::new(
                format!("this mnemonic requires operand at index {idx}, but one was not found"),
                9,
            );
            return Err(er);
        } else {
            break;
        }
    }
    if ops.len() == 2 {
        if let Some(err) = avx_size_chk(ins) {
            return Err(err);
        }
    }
    if let Some(err) = avx_forb_chk(ins, forb) {
        return Err(err);
    }
    Ok(())
}

fn avx_forb_chk(ins: &Instruction, forb: &[(AType, AType, AType)]) -> Option<Error> {
    let dst_t = if let Some(dst) = ins.dst() {
        dst.atype()
    } else {
        return None;
    };
    let src_t = if let Some(src) = ins.src() {
        src.atype()
    } else {
        return None;
    };
    let ssrc_t = if let Some(ssrc) = ins.ssrc() {
        ssrc.atype()
    } else {
        return None;
    };
    for f in forb {
        if (dst_t, src_t, ssrc_t) == *f {
            let er = Error::new(
                "you provided instruction, which has forbidden operand combination",
                7,
            );
            return Some(er);
        }
    }
    None
}

fn ot_chk(
    ins: &Instruction,
    ops: &[(&[AType], Optional)],
    forb: &[(AType, AType)],
    addt: &[Mnemonic],
) -> Result<(), Error> {
    if let Some(err) = addt_chk(ins, addt) {
        return Err(err);
    }
    if ops.is_empty() && !ins.is_empty() {
        let er = Error::new(
            "this mnemonic does not accept any operand, but you tried to use one",
            9,
        );
        return Err(er);
    }
    for (idx, allowed) in ops.iter().enumerate() {
        if let Some(op) = ins.get(idx) {
            if let Some(err) = type_check(&op, allowed.0, idx) {
                return Err(err);
            }
        } else if allowed.1 == Optional::Needed {
            let er = Error::new(
                format!("this mnemonic requires operand at index {idx}, but one was not found"),
                9,
            );
            return Err(er);
        } else {
            break;
        }
    }
    if ops.len() == 2 {
        if let Some(err) = size_chk(ins) {
            let mut b = false;
            for o in ops {
                for o in o.0 {
                    if let AType::Register(_, true) = o {
                        b = true;
                        break;
                    }
                }
            }
            if !b {
                return Err(err);
            }
        }
    }
    if let Some(err) = forb_chk(ins, forb) {
        return Err(err);
    }
    Ok(())
}

fn forb_chk(ins: &Instruction, forb: &[(AType, AType)]) -> Option<Error> {
    let dst_t = if let Some(dst) = ins.dst() {
        dst.atype()
    } else {
        return None;
    };
    let src_t = if let Some(src) = ins.src() {
        src.atype()
    } else {
        return None;
    };
    for f in forb {
        if (dst_t, src_t) == *f {
            let er = Error::new(
                "you provided instruction, which has forbidden operand combination",
                7,
            );
            return Some(er);
        }
    }
    None
}

fn type_check(operand: &Operand, accepted: &[AType], idx: usize) -> Option<Error> {
    if let Some(m) = operand.get_mem() {
        if m.addrsize() == Size::Word {
            let er = Error::new("currently it is forbidden to use 16-bit address size", 500);
            return Some(er);
        }
    }
    if accepted.iter().any(|s| s == &operand.atype()) {
        None
    } else {
        if let Operand::Imm(imm) = operand {
            if accepted.contains(&AType::Immediate(imm.signed_size(), false)) {
                return None;
            }
        }
        let er = Error::new(
            if operand.size() == Size::Qword {
                format!("operand at index {idx} has invalid type of {}. consider setting bits parameter to 64 as this could fix the issue.", operand.atype())
            } else {
                format!(
                    "operand at index {idx} has invalid type of {}",
                    operand.atype()
                )
            },
            8,
        );
        Some(er)
    }
}
fn avx_size_chk(ins: &Instruction) -> Option<Error> {
    let dst = ins.dst().unwrap();
    let src = ins.src().unwrap();

    // should work (i hope so)
    match (dst.atype(), src.atype()) {
        (AType::Register(r0, _), AType::Immediate(s1, _)) => {
            if s1 <= r0.size() {
                None
            } else if !ins.mnemonic.allows_diff_size(Some(r0.size()), Some(s1)) {
                let er = Error::new("you tried to use immediate which is too large", 8);
                Some(er)
            } else {
                None
            }
        }
        (AType::Memory(s0, _, _), AType::Immediate(s1, _)) => {
            if s1 <= s0 {
                None
            } else if !ins.mnemonic.allows_diff_size(Some(s0), Some(s1)) {
                let er = Error::new("you tried to use immediate which is too large", 8);
                Some(er)
            } else {
                None
            }
        }
        (AType::Memory(_, _, _), AType::Memory(_, _, _)) => {
            let er = Error::new("combination of memory and memory is forbidden", 8);
            Some(er)
        }
        (AType::Register(r0, _), AType::Register(r1, _)) => {
            let s0 = r0.size();
            let s1 = r1.size();
            if let Some(ssrc) = ins.ssrc() {
                if s1 == s0 && ssrc.size() == s0 {
                    None
                } else {
                    let er = Error::new("dst operand has invalid type", 8);
                    Some(er)
                }
            } else if s1 == s0 {
                None
            } else if !ins.mnemonic.allows_diff_size(Some(s0), Some(s1)) {
                let er = Error::new("dst operand has invalid type", 8);
                Some(er)
            } else {
                None
            }
        }

        _ => None,
    }
}
fn size_chk(ins: &Instruction) -> Option<Error> {
    let dst = ins.dst().unwrap();
    let src = ins.src().unwrap();

    if let Operand::Register(r) = dst {
        if r.is_ctrl_reg() {
            return None;
        }
    }
    if let Operand::Register(r) = src {
        if r.is_ctrl_reg() {
            return None;
        }
    }
    // should work (i hope so)
    match (dst.atype(), src.atype()) {
        (AType::Register(r0, _), AType::Immediate(s1, _)) => {
            if s1 <= r0.size() {
                None
            } else if !ins.mnemonic.allows_diff_size(Some(r0.size()), Some(s1)) {
                let er = Error::new("you tried to use immediate which is too large", 8);
                Some(er)
            } else {
                None
            }
        }
        (AType::Memory(s0, _, _), AType::Immediate(s1, _)) => {
            if s1 <= s0 {
                None
            } else if !ins.mnemonic.allows_diff_size(Some(s0), Some(s1)) {
                let er = Error::new("you tried to use immediate which is too large", 8);
                Some(er)
            } else {
                None
            }
        }
        (AType::Memory(_, _, _), AType::Memory(_, _, _)) => {
            let er = Error::new("combination of memory and memory is forbidden", 8);
            Some(er)
        }
        (AType::Register(r0, f0), AType::Register(r1, f1)) => {
            if f0 || f1 {
                return None;
            }

            let s0 = r0.size();
            let s1 = r1.size();
            let g0 = r0.purpose();
            let g1 = r1.purpose();
            if s1 == s0
                || ((g0 == RPurpose::Dbg
                    || g0 == RPurpose::Ctrl
                    || g0 == RPurpose::Sgmnt
                    || g0 == RPurpose::Mmx
                    || s0 == Size::Xword
                    || s0 == Size::Yword)
                    || (g1 == RPurpose::Dbg
                        || g1 == RPurpose::Ctrl
                        || g1 == RPurpose::Sgmnt
                        || g1 == RPurpose::Mmx
                        || s1 == Size::Xword
                        || s1 == Size::Yword))
            {
                None
            } else if !ins.mnemonic.allows_diff_size(Some(s0), Some(s1)) {
                let er = Error::new("dst operand has invalid type", 8);
                Some(er)
            } else {
                None
            }
        }

        _ => None,
    }
}

fn addt_chk(ins: &Instruction, accpt_addt: &[Mnemonic]) -> Option<Error> {
    if let Some(addt) = ins.get_addt() {
        if !find_bool(accpt_addt, &addt) {
            let er = Error::new("usage of forbidden additional mnemonic", 6);
            return Some(er);
        }
    }
    None
}

fn find_bool(addts: &[Mnemonic], searched: &Mnemonic) -> bool {
    for addt in addts {
        if searched == addt {
            return true;
        }
    }
    false
}
