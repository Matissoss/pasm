// pasm - src/pre/chk.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::pre::chkn;

use crate::shr::{
    ast::{Instruction, Operand, AST},
    atype::*,
    error::Error,
    ins::Mnemonic,
    reg::Purpose as RPurpose,
    size::Size,
};

pub fn check_ast(file: &AST) -> Option<Vec<(String, Vec<Error>)>> {
    let mut errors: Vec<(String, Vec<Error>)> = Vec::new();

    for section in &file.sections {
        for label in &section.content {
            let chk_ins: fn(&Instruction) -> Result<(), Error> = match label.attributes.get_bits() {
                64 => check_ins64bit,
                _ => check_ins32bit,
            };
            let mut errs = Vec::new();
            for inst in &label.content {
                if let Err(mut err) = chk_ins(inst) {
                    err.set_line(inst.line);
                    errs.push(err);
                }
            }
            if !errs.is_empty() {
                errors.push((label.name.to_string(), errs));
            }
        }
    }

    if errors.is_empty() {
        None
    } else {
        Some(errors)
    }
}

fn check_ins32bit(ins: &Instruction) -> Result<(), Error> {
    use Mnemonic::*;
    if ins.needs_rex() {
        let mut er = Error::new(
            "you tried to use instruction that requires REX prefix, but bits != 64",
            10,
        );
        er.set_line(ins.line);
        return Err(er);
    } else if ins.needs_evex() {
        let mut er = Error::new(
            "you tried to use instruction that requires EVEX prefix, but bits != 64",
            10,
        );
        er.set_line(ins.line);
        return Err(er);
    }
    match ins.mnem {
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
                    .pushop(&[R8, R16, R32, R64, SR, CR, DR, M8, M16, M32], true)
                    .pushop(
                        &[R8, R16, R32, SR, CR, DR, M8, M16, M32, I8, I16, I32],
                        true,
                    )
                    .set_forb(&[
                        [MA, MA],
                        [R32, SR],
                        [M32, SR],
                        [M8, SR],
                        [R8, SR],
                        [SR, R8],
                        [SR, IA],
                        [SR, M8],
                        [CR, IA],
                        [CR, R8],
                        [CR, R16],
                        [R16, CR],
                        [DR, IA],
                        [DR, R8],
                        [DR, R16],
                        [R16, DR],
                        [R8, DR],
                        [DR, MA],
                        [MA, DR],
                        [R8, DR],
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
            .pushop(&[R8, R16, R32, M8, M16, M32], true)
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
        Mnemonic::MOVQ | MOVSTRQ | SCASQ | STOSQ => {
            let mut er = Error::new(
                "you tried to use instruction that is invalid when bits != 64",
                10,
            );
            er.set_line(ins.line);
            Err(er)
        }
        _ => shr_chk(ins),
    }
}

fn check_ins64bit(ins: &Instruction) -> Result<(), Error> {
    use Mnemonic::*;
    match ins.mnem {
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
            .pushop(&[R16, R32, R64], true)
            .pushop(&[R16, R32, R64, M16, M32, M64], true)
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
                .pushop(&[R8, R16, R32, R64, SR, CR, DR, M8, M16, M32, M64], true)
                .pushop(
                    &[
                        R8, R16, R32, R64, SR, CR, DR, M8, M16, M32, M64, I8, I16, I32, I64,
                    ],
                    true,
                )
                .set_forb(&[
                    [MA, MA],
                    [R32, SR],
                    [M32, SR],
                    [M8, SR],
                    [R8, SR],
                    [SR, R32],
                    [SR, R8],
                    [SR, IA],
                    [SR, M8],
                    [CR, IA],
                    [CR, R8],
                    [CR, R16],
                    [CR, R32],
                    [R16, CR],
                    [DR, IA],
                    [DR, R8],
                    [DR, R16],
                    [DR, R32],
                    [R16, DR],
                    [R8, DR],
                    [DR, MA],
                    [MA, DR],
                    [R8, DR],
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
                .pushop(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .pushop(&[R8, R16, R32, R64, M8, M16, M32, M64], true)
                .set_forb(&[[MA, MA]])
                .set_addt(&[LOCK])
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
        _ => shr_chk(ins),
    }
}

pub fn shr_chk(ins: &Instruction) -> Result<(), Error> {
    use Mnemonic::*;
    match ins.mnem {
        LGDT | LIDT => ot_chk(ins, &[(&[M16], Optional::Needed)], &[], &[]),

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
        PREFETCHW | PREFETCH0 | PREFETCH1 | PREFETCH2 | PREFETCHA => {
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
        | UIRET | WAIT | FWAIT | WBINVD | WRMSR | WRPKRU => ot_chk(ins, &[], &[], &[]),
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
        LLDT | LMSW => ot_chk(ins, &[(&[R16, M16], Optional::Needed)], &[], &[]),

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
        | XSUSLDTRK | XTEST => ot_chk(ins, &[], &[], &[]),

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
        //  ###   #   #  #   #
        // #   #  #   #   # #
        // #   #   # #     #
        // #####   # #    # #
        // #   #    #    #   #
        // AVX chk

        // idk derived
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
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
                (&[XMM, YMM, M256, M128, I8], Optional::Needed),
            ],
            &[(XMM, MA, MA), (YMM, MA, MA)],
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
        Mnemonic::VCVTSS2SI | Mnemonic::VCVTTSS2SI => ot_chk(
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
                .pushop(&[XMM, YMM, ZMM], true)
                .pushop(&[XMM, YMM, ZMM], true)
                .pushop(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .set_avx512()
                .set_mask_perm()
                .check(ins)
        }
        VADDSH => {
            use chkn::*;
            CheckAPI::<3>::new()
                .pushop(&[XMM], true)
                .pushop(&[XMM], true)
                .pushop(&[XMM, M16], true)
                .set_avx512()
                .set_mask_perm()
                .check(ins)
        }
        VBLENDMPD => {
            use chkn::*;
            CheckAPI::<3>::new()
                .pushop(&[XMM, YMM, ZMM], true)
                .pushop(&[XMM, YMM, ZMM], true)
                .pushop(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .set_avx512()
                .set_mask_perm()
                .check(ins)
        }
        VBLENDMPS => {
            use chkn::*;
            CheckAPI::<3>::new()
                .pushop(&[XMM, YMM, ZMM], true)
                .pushop(&[XMM, YMM, ZMM], true)
                .pushop(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .set_avx512()
                .set_mask_perm()
                .check(ins)
        }
        VALIGND => {
            use chkn::*;
            CheckAPI::<4>::new()
                .pushop(&[XMM, YMM, ZMM], true)
                .pushop(&[XMM, YMM, ZMM], true)
                .pushop(&[XMM, YMM, ZMM, M128, M256, M512, MBCST32], true)
                .pushop(&[I8], true)
                .set_avx512()
                .set_mask_perm()
                .check(ins)
        }
        VALIGNQ => {
            use chkn::*;
            CheckAPI::<4>::new()
                .pushop(&[XMM, YMM, ZMM], true)
                .pushop(&[XMM, YMM, ZMM], true)
                .pushop(&[XMM, YMM, ZMM, M128, M256, M512, MBCST64], true)
                .pushop(&[I8], true)
                .set_avx512()
                .set_mask_perm()
                .check(ins)
        }
        VBROADCASTF32X2 => {
            use chkn::*;
            CheckAPI::<2>::new()
                .pushop(&[YMM, ZMM], true)
                .pushop(&[XMM, M64], true)
                .set_avx512()
                .set_mode(CheckMode::AVX)
                .set_mask_perm()
                .check(ins)
        }
        VBROADCASTF32X4 | VBROADCASTF64X2 => {
            use chkn::*;
            CheckAPI::<2>::new()
                .pushop(&[YMM, ZMM], true)
                .pushop(&[XMM, M128], true)
                .set_avx512()
                .set_mode(CheckMode::AVX)
                .set_mask_perm()
                .check(ins)
        }
        VBROADCASTF32X8 | VBROADCASTF64X4 => {
            use chkn::*;
            CheckAPI::<2>::new()
                .pushop(&[ZMM], true)
                .pushop(&[M256], true)
                .set_avx512()
                .set_mode(CheckMode::AVX)
                .set_mask_perm()
                .check(ins)
        }
        VBCSTNEBF162PS | VBCSTNESH2PS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .pushop(&[XMM, YMM], true)
                .pushop(&[M16], true)
                .check(ins)
        }
        VCOMISH => {
            use chkn::*;
            CheckAPI::<2>::new()
                .pushop(&[XMM], true)
                .pushop(&[XMM, M16], true)
                .set_avx512()
                .check(ins)
        }
        VCOMPRESSPD | VCOMPRESSPS => {
            use chkn::*;
            CheckAPI::<2>::new()
                .pushop(&[XMM, YMM, ZMM, M128, M256, M512], true)
                .pushop(&[XMM, YMM, ZMM], true)
                .set_mask_perm()
                .set_avx512()
                .check(ins)
        }
        VCMPSH => {
            use chkn::*;
            CheckAPI::<4>::new()
                .pushop(&[K], true)
                .pushop(&[XMM], true)
                .pushop(&[XMM, M128, M16], true)
                .pushop(&[I8], true)
                .set_mask_perm()
                .set_avx512()
                .check(ins)
        }
        VCMPPH => {
            use chkn::*;
            CheckAPI::<4>::new()
                .pushop(&[K], true)
                .pushop(&[XMM, YMM, ZMM], true)
                .pushop(&[XMM, YMM, ZMM, M128, M256, M512, MBCST16], true)
                .pushop(&[I8], true)
                .set_mask_perm()
                .set_avx512()
                .check(ins)
        }

        _ => {
            let mut er = Error::new(
                "internal error: instruction does not have entry in check layer",
                500,
            );
            er.set_line(ins.line);
            Err(er)
        }
    }
}

// Utils

// Legacy check API
#[derive(PartialEq)]
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
    if ops.is_empty() && !ins.operands.is_empty() {
        let mut er = Error::new(
            "this mnemonic does not accept any operand, but you tried to use one",
            9,
        );
        er.set_line(ins.line);
        return Err(er);
    }
    for (idx, allowed) in ops.iter().enumerate() {
        if let Some(op) = ins.get_opr(idx) {
            if let Some(err) = type_check(op, allowed.0, idx) {
                return Err(err);
            }
        } else if allowed.1 == Optional::Needed {
            let mut er = Error::new(
                format!("this mnemonic requires operand at index {idx}, but one was not found"),
                9,
            );
            er.set_line(ins.line);
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
    if ops.is_empty() && !ins.operands.is_empty() {
        let mut er = Error::new(
            "this mnemonic does not accept any operand, but you tried to use one",
            9,
        );
        er.set_line(ins.line);
        return Err(er);
    }
    for (idx, allowed) in ops.iter().enumerate() {
        if let Some(op) = ins.get_opr(idx) {
            if let Some(err) = type_check(op, allowed.0, idx) {
                return Err(err);
            }
        } else if allowed.1 == Optional::Needed {
            let mut er = Error::new(
                format!("this mnemonic requires operand at index {idx}, but one was not found"),
                9,
            );
            er.set_line(ins.line);
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
    let ssrc_t = if let Some(ssrc) = ins.src2() {
        ssrc.atype()
    } else {
        return None;
    };
    for f in forb {
        if (dst_t, src_t, ssrc_t) == *f {
            let mut er = Error::new(
                "you provided instruction, which has forbidden operand combination",
                7,
            );
            er.set_line(ins.line);
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
    if ops.is_empty() && !ins.operands.is_empty() {
        let mut er = Error::new(
            "this mnemonic does not accept any operand, but you tried to use one",
            9,
        );
        er.set_line(ins.line);
        return Err(er);
    }
    for (idx, allowed) in ops.iter().enumerate() {
        if let Some(op) = ins.get_opr(idx) {
            if let Some(err) = type_check(op, allowed.0, idx) {
                return Err(err);
            }
        } else if allowed.1 == Optional::Needed {
            let mut er = Error::new(
                format!("this mnemonic requires operand at index {idx}, but one was not found"),
                9,
            );
            er.set_line(ins.line);
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
            let mut er = Error::new(
                "you provided instruction, which has forbidden operand combination",
                7,
            );
            er.set_line(ins.line);
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
            if accepted.contains(&AType::Immediate(imm.size(), false)) {
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
            } else if !ins.mnem.allows_diff_size(Some(r0.size()), Some(s1)) {
                let mut er = Error::new("you tried to use immediate which is too large", 8);
                er.set_line(ins.line);
                Some(er)
            } else {
                None
            }
        }
        (AType::Memory(s0, _, _), AType::Immediate(s1, _)) => {
            if s1 <= s0 {
                None
            } else if !ins.mnem.allows_diff_size(Some(s0), Some(s1)) {
                let mut er = Error::new("you tried to use immediate which is too large", 8);
                er.set_line(ins.line);
                Some(er)
            } else {
                None
            }
        }
        (AType::Memory(_, _, _), AType::Memory(_, _, _)) => {
            let mut er = Error::new("combination of memory and memory is forbidden", 8);
            er.set_line(ins.line);
            Some(er)
        }
        (AType::Register(r0, _), AType::Register(r1, _)) => {
            let s0 = r0.size();
            let s1 = r1.size();
            if let Some(ssrc) = ins.src2() {
                if s1 == s0 && ssrc.size() == s0 {
                    None
                } else {
                    let mut er = Error::new("dst operand has invalid type", 8);
                    er.set_line(ins.line);
                    Some(er)
                }
            } else if s1 == s0 {
                None
            } else if !ins.mnem.allows_diff_size(Some(s0), Some(s1)) {
                let mut er = Error::new("dst operand has invalid type", 8);
                er.set_line(ins.line);
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
            } else if !ins.mnem.allows_diff_size(Some(r0.size()), Some(s1)) {
                let mut er = Error::new("you tried to use immediate which is too large", 8);
                er.set_line(ins.line);
                Some(er)
            } else {
                None
            }
        }
        (AType::Memory(s0, _, _), AType::Immediate(s1, _)) => {
            if s1 <= s0 {
                None
            } else if !ins.mnem.allows_diff_size(Some(s0), Some(s1)) {
                let mut er = Error::new("you tried to use immediate which is too large", 8);
                er.set_line(ins.line);
                Some(er)
            } else {
                None
            }
        }
        (AType::Memory(_, _, _), AType::Memory(_, _, _)) => {
            let mut er = Error::new("combination of memory and memory is forbidden", 8);
            er.set_line(ins.line);
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
            } else if !ins.mnem.allows_diff_size(Some(s0), Some(s1)) {
                let mut er = Error::new("dst operand has invalid type", 8);
                er.set_line(ins.line);
                Some(er)
            } else {
                None
            }
        }

        _ => None,
    }
}

fn addt_chk(ins: &Instruction, accpt_addt: &[Mnemonic]) -> Option<Error> {
    if let Some(addt) = &ins.addt {
        if !find_bool(accpt_addt, addt) {
            let mut er = Error::new("usage of forbidden additional mnemonic", 6);
            er.set_line(ins.line);
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
