// rasmx86_64 - src/pre/chk.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::core::rex::gen_rex;
use crate::shr::{
    ast::{Instruction, Operand, AST},
    atype::*,
    error::RASMError,
    ins::Mnemonic as Mnm,
    reg::{Purpose as RPurpose, Register},
    size::Size,
};

use AType::*;

pub fn check_ast(file: &AST) -> Option<Vec<(String, Vec<RASMError>)>> {
    let mut errors: Vec<(String, Vec<RASMError>)> = Vec::new();

    for section in &file.sections {
        for label in &section.content {
            let chk_ins: fn(&Instruction) -> Option<RASMError> = match label.bits {
                64 => check_ins64bit,
                _ => check_ins32bit,
            };
            let mut errs = Vec::new();
            for inst in &label.inst {
                if let Some(mut err) = chk_ins(inst) {
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

fn check_ins32bit(ins: &Instruction) -> Option<RASMError> {
    use Mnm::*;
    if gen_rex(ins, false).is_some() {
        return Some(RASMError::no_tip(
            Some(ins.line),
            Some("Instruction needs rex prefix, which is forbidden in protected/compat. mode (bits 32)"),
        ));
    }
    match ins.mnem {
        Mnm::CMOVA
        | Mnm::CMOVB
        | Mnm::CMOVC
        | Mnm::CMOVE
        | Mnm::CMOVG
        | Mnm::CMOVL
        | Mnm::CMOVO
        | Mnm::CMOVP
        | Mnm::CMOVS
        | Mnm::CMOVZ
        | Mnm::CMOVAE
        | Mnm::CMOVBE
        | Mnm::CMOVLE
        | Mnm::CMOVGE
        | Mnm::CMOVNA
        | Mnm::CMOVNB
        | Mnm::CMOVNC
        | Mnm::CMOVNE
        | Mnm::CMOVNG
        | Mnm::CMOVNL
        | Mnm::CMOVNO
        | Mnm::CMOVNP
        | Mnm::CMOVNS
        | Mnm::CMOVNZ
        | Mnm::CMOVPE
        | Mnm::CMOVPO
        | Mnm::CMOVNBE
        | Mnm::CMOVNLE
        | Mnm::CMOVNGE
        | Mnm::CMOVNAE => ot_chk(
            ins,
            &[
                (&[R16, R32], Optional::Needed),
                (&[R16, R32, M16, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),

        Mnm::PUSH => ot_chk(
            ins,
            &[(&[R16, R32, M16, M32, I8, I16, I32, SR], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::POP => ot_chk(
            ins,
            &[(&[R16, R32, M16, M32, SR], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::MOV => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, M8, M16, M32, SR, CR, DR], Optional::Needed),
                (
                    &[R8, R16, R32, M8, M16, M32, I8, I16, I32, SR, CR, DR],
                    Optional::Needed,
                ),
            ],
            &[
                (MA, MA),
                (R32, SR),
                (M32, SR),
                (M8, SR),
                (R8, SR),
                (SR, R32),
                (SR, R8),
                (SR, IA),
                (SR, M32),
                (SR, M8),
                (CR, IA),
                (CR, R8),
                (CR, R16),
                (R16, CR),
                (R8, CR),
                (CR, MA),
                (MA, CR),
                (DR, IA),
                (DR, R8),
                (DR, R16),
                (DR, R32),
                (R16, DR),
                (R8, DR),
                (DR, MA),
                (MA, DR),
                (R8, DR),
                (DR, MA),
                (MA, DR),
                (SR, CR),
                (SR, DR),
                (CR, SR),
                (CR, DR),
                (DR, SR),
                (SR, SR),
                (DR, DR),
                (CR, CR),
            ],
            &[],
        ),
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
        Mnm::SUB | Mnm::ADD | Mnm::AND | Mnm::OR | Mnm::XOR | Mnm::ADC | SBB => ot_chk(
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
        Mnm::IMUL => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, M8, M16, M32], Optional::Needed),
                (&[R16, R32, M16, M32], Optional::Optional),
                (&[I8, I16, I32], Optional::Optional),
            ],
            &[(MA, MA)],
            &[],
        ),
        Mnm::SAL | Mnm::SHL | Mnm::SHR | Mnm::SAR | ROL | RCL | ROR | RCR => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, M8, M16, M32], Optional::Needed),
                (&[ExtendedRegister(Register::CL), I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::TEST => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, M8, M16, M32], Optional::Needed),
                (&[I8, I16, I32, R8, R16, R32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::DIV | Mnm::IDIV | Mnm::MUL => ot_chk(
            ins,
            &[(&[R8, R16, R32, M8, M16, M32], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::DEC | Mnm::INC | Mnm::NEG | Mnm::NOT => ot_chk(
            ins,
            &[(&[R8, R16, R32, M8, M16, M32], Optional::Needed)],
            &[],
            &[LOCK],
        ),
        
        Mnm::JMP | Mnm::CALL => ot_chk(
            ins,
            &[(&[AType::Symbol, R32, R16, M32, M16], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::LEA => ot_chk(
            ins,
            &[
                (&[R16, R32], Optional::Needed),
                (&[AType::Symbol], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::SYSCALL | Mnm::RET | Mnm::NOP | Mnm::POPF | Mnm::POPFD | Mnm::PUSHF | Mnm::PUSHFD => {
            ot_chk(ins, &[], &[], &[])
        }
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
        Mnm::BT => ot_chk(
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
        CMPSTRB | CMPSTRW | CMPSTRD | SCASB | SCASW | SCASD => ot_chk(ins, &[], &[], &[REPE, REPZ, REPNE, REPNZ]),
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
                (&[I8, ExtendedRegister(Register::CL)], Optional::Needed),
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
        Mnm::MOVD => ot_chk(
            ins,
            &[
                (&[MMX, XMM, R32, M32], Optional::Needed),
                (&[MMX, XMM, R32, M32], Optional::Needed),
            ],
            &[(M32, M32), (R32, R32), (MMX, MMX), (XMM, MMX), (MMX, XMM)],
            &[],
        ),
        Mnm::MOVQ | MOVSTRQ | SCASQ | STOSQ => Some(RASMError::no_tip(
            Some(ins.line),
            Some("Instruction unsupported in 32-bit mode"),
        )),
        _ => shr_chk(ins),
    }
}

fn check_ins64bit(ins: &Instruction) -> Option<RASMError> {
    use Mnm::*;
    match ins.mnem {
        Mnm::CMOVA
        | Mnm::CMOVB
        | Mnm::CMOVC
        | Mnm::CMOVE
        | Mnm::CMOVG
        | Mnm::CMOVL
        | Mnm::CMOVO
        | Mnm::CMOVP
        | Mnm::CMOVS
        | Mnm::CMOVZ
        | Mnm::CMOVAE
        | Mnm::CMOVBE
        | Mnm::CMOVLE
        | Mnm::CMOVGE
        | Mnm::CMOVNA
        | Mnm::CMOVNB
        | Mnm::CMOVNC
        | Mnm::CMOVNE
        | Mnm::CMOVNG
        | Mnm::CMOVNL
        | Mnm::CMOVNO
        | Mnm::CMOVNP
        | Mnm::CMOVNS
        | Mnm::CMOVNZ
        | Mnm::CMOVPE
        | Mnm::CMOVPO
        | Mnm::CMOVNBE
        | Mnm::CMOVNLE
        | Mnm::CMOVNGE
        | Mnm::CMOVNAE => ot_chk(
            ins,
            &[
                (&[R16, R32, R64], Optional::Needed),
                (&[R16, R32, R64, M16, M32, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::CLFLUSH => ot_chk(ins, &[(&[M8], Optional::Needed)], &[], &[]),
        Mnm::PAUSE | Mnm::LFENCE | Mnm::MFENCE => ot_chk(ins, &[], &[], &[]),
        Mnm::PUSH => ot_chk(
            ins,
            &[(&[R16, R64, M16, M64, I8, I16, I32, SR], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::POP => ot_chk(
            ins,
            &[(&[R16, R64, M16, M64, SR], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::MOV => ot_chk(
            ins,
            &[
                (
                    &[R8, R16, R32, R64, M8, M16, M32, M64, SR, CR, DR],
                    Optional::Needed,
                ),
                (
                    &[
                        R8, R16, R32, R64, M8, M16, M32, M64, I8, I16, I32, I64, SR, CR, DR,
                    ],
                    Optional::Needed,
                ),
            ],
            &[
                (MA, MA),
                (R32, SR),
                (M32, SR),
                (M8, SR),
                (R8, SR),
                (SR, R32),
                (SR, R8),
                (SR, IA),
                (SR, M32),
                (SR, M8),
                (CR, IA),
                (CR, R8),
                (CR, R16),
                (CR, R32),
                (R16, CR),
                (DR, IA),
                (DR, R8),
                (DR, R16),
                (DR, R32),
                (R16, DR),
                (R8, DR),
                (DR, MA),
                (MA, DR),
                (R8, DR),
                (DR, MA),
                (MA, DR),
            ],
            &[],
        ),
        XCHG => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed),
                (&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed),
            ],
            &[(MA, MA)],
            &[LOCK],
        ),
        Mnm::CMP => ot_chk(
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
        Mnm::SUB | Mnm::ADD | Mnm::AND | Mnm::OR | Mnm::XOR | ADC | SBB => ot_chk(
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
        Mnm::IMUL => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed),
                (&[R16, R32, R64, M16, M32, M64], Optional::Optional),
                (&[I8, I16, I32], Optional::Optional),
            ],
            &[(MA, MA)],
            &[],
        ),
        Mnm::SAL | Mnm::SHL | Mnm::SHR | Mnm::SAR | ROL | RCL | ROR | RCR => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed),
                (&[ExtendedRegister(Register::CL), I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::TEST => ot_chk(
            ins,
            &[
                (&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed),
                (&[I8, I16, I32, R8, R16, R32, R64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::DIV | Mnm::IDIV | Mnm::MUL => ot_chk(
            ins,
            &[(&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::DEC | Mnm::INC | Mnm::NEG | Mnm::NOT => ot_chk(
            ins,
            &[(&[R8, R16, R32, R64, M8, M16, M32, M64], Optional::Needed)],
            &[],
            &[LOCK],
        ),
        Mnm::JMP | Mnm::CALL => ot_chk(
            ins,
            &[(&[AType::Symbol, R64, M64], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::LEA => ot_chk(
            ins,
            &[
                (&[R16, R32, R64], Optional::Needed),
                (&[AType::Symbol, MA], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::SYSCALL | Mnm::RET | Mnm::NOP | Mnm::PUSHF | Mnm::POPF | Mnm::POPFQ | Mnm::PUSHFQ => {
            ot_chk(ins, &[], &[], &[])
        }
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
        CMPSTRB | CMPSTRW | CMPSTRD | CMPSTRQ | SCASB | SCASW | SCASD | SCASQ => ot_chk(ins, &[], &[], &[REPE, REPZ, REPNE, REPNZ]),
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
                (&[I8, ExtendedRegister(Register::CL)], Optional::Needed),
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

pub fn shr_chk(ins: &Instruction) -> Option<RASMError> {
    use Mnm::*;
    use Register::*;
    match ins.mnem {
        LGDT | LIDT => ot_chk(ins, &[
            (&[M16], Optional::Needed)
        ], &[], &[]),

        OUT => ot_chk(ins, &[
            (&[ExtendedRegister(DX), I8], Optional::Needed),
            (&[ExtendedRegister(AL), ExtendedRegister(AX), ExtendedRegister(EAX)], Optional::Needed), 
        ], &[], &[]),
        IN => ot_chk(ins, &[
            (&[ExtendedRegister(AL), ExtendedRegister(AX), ExtendedRegister(EAX)], Optional::Needed),
            (&[ExtendedRegister(DX), I8], Optional::Needed)
        ], &[], &[]),

        // instruction as "variable"
        BYTE | BYTELE | BYTEBE => ot_chk(ins, &[(&[I8], Optional::Needed)], &[], &[]),
        WORD | WORDLE | WORDBE => ot_chk(ins, &[(&[I8, I16], Optional::Needed)], &[], &[]),
        DWORD | DWORDLE | DWORDBE => ot_chk(ins, &[(&[I8, I16, I32], Optional::Needed)], &[], &[]),
        QWORD | QWORDLE | QWORDBE => {
            ot_chk(ins, &[(&[I8, I16, I32, I64], Optional::Needed)], &[], &[])
        }
        EMPTY => ot_chk(ins, &[(&[I8, I16], Optional::Needed)], &[], &[]),
        STRZ | ASCIIZ => ot_chk(ins, &[(&[ASTR], Optional::Needed)], &[], &[]),

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

        SFENCE | STAC | STC | STD | STI | STUI | SYSENTER
        | SYSEXIT | SYSRET | TESTUI | UD2 | UIRET | WAIT | FWAIT | WBINVD | WRMSR | WRPKRU => {
            ot_chk(ins, &[], &[], &[])
        }
        TPAUSE | UMWAIT => ot_chk(
            ins,
            &[
                (&[R32], Optional::Needed),
                (&[ExtendedRegister(Register::EDX)], Optional::Optional),
                (&[ExtendedRegister(Register::EAX)], Optional::Needed),
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
                (&[I8, ExtendedRegister(Register::CL)], Optional::Needed),
            ],
            &[],
            &[],
        ),

        LOOP
        | LOOPNE
        | LOOPE
        | Mnm::JA
        | Mnm::JB
        | Mnm::JC
        | Mnm::JO
        | Mnm::JP
        | Mnm::JS
        | Mnm::JAE
        | Mnm::JNAE
        | Mnm::JNBE
        | Mnm::JNGE
        | Mnm::JBE
        | Mnm::JNO
        | Mnm::JNP
        | Mnm::JPO
        | Mnm::JPE
        | Mnm::JNA
        | Mnm::JNL
        | Mnm::JNLE
        | Mnm::JNC
        | Mnm::JNB
        | Mnm::JE
        | Mnm::JNE
        | Mnm::JZ
        | Mnm::JNZ
        | Mnm::JL
        | Mnm::JLE
        | Mnm::JG
        | Mnm::JGE => ot_chk(ins, &[(&[AType::Symbol], Optional::Needed)], &[], &[]),
        Mnm::CPUID => ot_chk(ins, &[], &[], &[]),

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

        RDMSR | RDPKRU | RDPMC | RDTSC | RDTSCP | RSM | SAHF
        | SERIALIZE | SETSSBY => ot_chk(ins, &[], &[], &[]),
        RSTORSSP => ot_chk(ins, &[(&[M64], Optional::Needed)], &[], &[]),

        SETA | SETAE | SETB | SETBE | SETC | SETE | SETG | SETGE | SETL | SETLE | SETNA
        | SETNAE | SETNB | SETNBE | SETNC | SETNE | SETNG | SETNL | SETNGE | SETNLE | SETNO
        | SETNP | SETNS | SETNZ | SETO | SETP | SETPE | SETPO | SETS | SETZ => {
            ot_chk(ins, &[(&[crate::shr::atype::R8, M8], Optional::Needed)], &[], &[])
        }

        // norm-part6
        XABORT => ot_chk(ins, &[(&[I8], Optional::Needed)], &[], &[]),
        XACQUIRE | XRELEASE | XEND | XGETBV | XLAT | XLATB | XLATB64 | XRESLDTRK | XSETBV
        | XSUSLDTRK | XTEST => ot_chk(ins, &[], &[], &[]),

        XBEGIN => ot_chk(ins, &[(&[Symbol], Optional::Needed)], &[], &[]),
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
        Mnm::CMPSS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::UNPCKLPS | Mnm::UNPCKHPS => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),

        Mnm::CMPPS | Mnm::SHUFPS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::MOVHPS | Mnm::MOVLPS => ot_chk(
            ins,
            &[
                (&[XMM, M64], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[(M64, M64)],
            &[],
        ),
        Mnm::MOVLHPS | Mnm::MOVHLPS => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::MOVAPS | Mnm::MOVUPS => ot_chk(
            ins,
            &[
                (&[XMM, M128], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
            ],
            &[(M128, M128)],
            &[],
        ),
        Mnm::MOVSS => ot_chk(
            ins,
            &[
                (&[XMM, M32], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[(M32, M32)],
            &[],
        ),
        Mnm::SQRTSS
        | Mnm::ADDSS
        | Mnm::SUBSS
        | Mnm::DIVSS
        | Mnm::MULSS
        | Mnm::RCPSS
        | Mnm::RSQRTSS
        | Mnm::MINSS
        | Mnm::MAXSS => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M32], Optional::Needed)],
            &[],
            &[],
        ),

        Mnm::ADDPS
        | Mnm::SUBPS
        | Mnm::DIVPS
        | Mnm::MULPS
        | Mnm::RCPPS
        | Mnm::SQRTPS
        | Mnm::RSQRTPS
        | Mnm::MINPS
        | Mnm::MAXPS
        | Mnm::ORPS
        | Mnm::ANDPS
        | Mnm::XORPS => ot_chk(
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
        Mnm::MOVDQ2Q => ot_chk(
            ins,
            &[(&[MMX], Optional::Needed), (&[XMM], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::MOVMSKPD => ot_chk(
            ins,
            &[(&[R32, R64], Optional::Needed), (&[XMM], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::MOVLPD | Mnm::MOVHPD | Mnm::MOVSD => ot_chk(
            ins,
            &[
                (&[XMM, M64], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[(M64, M64)],
            &[],
        ),
        Mnm::MOVAPD | Mnm::MOVUPD | Mnm::MOVDQA => ot_chk(
            ins,
            &[
                (&[XMM, M128], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
            ],
            &[(M128, M128)],
            &[],
        ),
        Mnm::CMPSD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),

        Mnm::CMPPD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),

        Mnm::SQRTSD
        | Mnm::ADDSD
        | Mnm::SUBSD
        | Mnm::DIVSD
        | Mnm::MULSD
        | Mnm::MINSD
        | Mnm::COMISD
        | Mnm::UCOMISD
        | Mnm::MAXSD => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M64], Optional::Needed)],
            &[],
            &[],
        ),

        Mnm::ADDPD
        | Mnm::SUBPD
        | Mnm::DIVPD
        | Mnm::MULPD
        | Mnm::SQRTPD
        | Mnm::MINPD
        | Mnm::MAXPD
        | Mnm::ORPD
        | Mnm::ANDPD
        | Mnm::XORPD => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::MASKMOVDQU => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::MOVNTDQ | Mnm::MOVNTPD => ot_chk(
            ins,
            &[(&[M128], Optional::Needed), (&[XMM], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::MOVNTI => ot_chk(
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
        Mnm::EMMS => ot_chk(ins, &[], &[], &[]),
        Mnm::MOVD => ot_chk(
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
        Mnm::MOVQ => ot_chk(
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
        Mnm::PSLLW
        | Mnm::PSLLD
        | Mnm::PSLLQ
        | Mnm::PSRLW
        | Mnm::PSRLD
        | Mnm::PSRLQ
        | Mnm::PSRAD
        | Mnm::PSRAW => ot_chk(
            ins,
            &[
                (&[MMX, XMM], Optional::Needed),
                (&[I8, MMX, XMM, M64, M128], Optional::Needed),
            ],
            &[(XMM, MMX), (MMX, XMM), (MMX, M128), (XMM, M64)],
            &[],
        ),
        Mnm::PADDB
        | Mnm::PADDW
        | Mnm::PADDD
        | Mnm::PADDQ
        | Mnm::PADDSB
        | Mnm::PADDSW
        | Mnm::PADDUSB
        | Mnm::PADDUSW
        | Mnm::PSUBB
        | Mnm::PSUBW
        | Mnm::PSUBSB
        | Mnm::PSUBSW
        | Mnm::PSUBUSB
        | Mnm::PSUBUSW
        | Mnm::PMULHW
        | Mnm::PMULLW
        | Mnm::PMADDWD
        | Mnm::PCMPGTB
        | Mnm::PCMPGTW
        | Mnm::PCMPGTD
        | Mnm::PCMPEQB
        | Mnm::PCMPEQW
        | Mnm::PCMPEQD
        | Mnm::PACKSSWB
        | Mnm::PACKSSDW
        | Mnm::PACKUSWB
        | Mnm::PUNPCKLBW
        | Mnm::PUNPCKLWD
        | Mnm::PUNPCKLDQ
        | Mnm::PUNPCKHBW
        | Mnm::PUNPCKHWD
        | Mnm::PUNPCKHDQ
        | Mnm::PAND
        | Mnm::PANDN
        | Mnm::POR
        | Mnm::PXOR
        | Mnm::PSUBD => ot_chk(
            ins,
            &[
                (&[MMX, XMM], Optional::Needed),
                (&[MMX, XMM, M64, M128], Optional::Needed),
            ],
            &[(XMM, MMX), (MMX, XMM), (XMM, M64), (MMX, M128)],
            &[],
        ),
        Mnm::PMULUDQ | Mnm::PSUBQ => ot_chk(
            ins,
            &[
                (&[MMX, XMM], Optional::Needed),
                (&[MMX, M64, XMM, M128], Optional::Needed),
            ],
            &[(MMX, XMM), (XMM, MMX), (XMM, M64), (MMX, M128)],
            &[],
        ),
        Mnm::PSHUFLW | Mnm::PSHUFHW | Mnm::PSHUFD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::PSLLDQ | Mnm::PSRLDQ => ot_chk(
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
        Mnm::ADDSUBPD
        | Mnm::ADDSUBPS
        | Mnm::HADDPD
        | Mnm::HADDPS
        | Mnm::HSUBPS
        | Mnm::HSUBPD
        | Mnm::MOVSLDUP
        | Mnm::MOVSHDUP => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),

        // weird one
        Mnm::LDDQU => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::MOVDDUP => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M64], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::MONITOR | Mnm::MWAIT | Mnm::MFENCE | Mnm::LFENCE | Mnm::CLFLUSH => {
            ot_chk(ins, &[], &[], &[])
        }

        // ##### ##### #####  #####   ####
        // #     #     #      #           #
        // ##### ##### #####  #####   ####
        //     #     #     #  #           #
        // ##### ##### #####  #####   ####
        // (SSSE 3)
        Mnm::PABSW
        | Mnm::PABSD
        | Mnm::PABSB
        | Mnm::PSIGNW
        | Mnm::PSIGND
        | Mnm::PSIGNB
        | Mnm::PHSUBW
        | Mnm::PHSUBD
        | Mnm::PHADDW
        | Mnm::PHADDD
        | Mnm::PSHUFB
        | Mnm::PHSUBSW
        | Mnm::PHADDSW
        | Mnm::PMULHRSW
        | Mnm::PMADDUBSW => ot_chk(
            ins,
            &[
                (&[MMX, XMM], Optional::Needed),
                (&[MMX, XMM, M64, M128], Optional::Needed),
            ],
            &[(MMX, XMM), (XMM, M64), (XMM, MMX), (MMX, M128)],
            &[],
        ),

        Mnm::PALIGNR => ot_chk(
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
        Mnm::PINSRB => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[R32, M8], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::PINSRQ => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[R64, M64], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::PINSRD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[R32, M32], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::PEXTRB => ot_chk(
            ins,
            &[
                (&[R32, R64, M32, M64], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::PEXTRD => ot_chk(
            ins,
            &[
                (&[R32, M32], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::PEXTRQ => ot_chk(
            ins,
            &[
                (&[R64, M64], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::PEXTRW => ot_chk(
            ins,
            &[
                (&[M16], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::PTEST
        | Mnm::PMAXSB
        | Mnm::PMAXSD
        | Mnm::PMINSD
        | Mnm::PMINSB
        | Mnm::PMINUW
        | Mnm::PMULDQ
        | Mnm::PMULLD
        | Mnm::PCMPEQQ
        | Mnm::PCMPGTQ
        | Mnm::BLENDVPS
        | Mnm::BLENDVPD
        | Mnm::PACKUSDW => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::POPCNT => ot_chk(
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
        Mnm::MOVNTDQA => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::EXTRACTPS => ot_chk(
            ins,
            &[
                (&[R32, R64, M32], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::ROUNDSS | Mnm::ROUNDSD | Mnm::INSERTPS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::DPPS
        | Mnm::DPPD
        | Mnm::BLENDPS
        | Mnm::BLENDPD
        | Mnm::PBLENDW
        | Mnm::ROUNDPS
        | Mnm::ROUNDPD
        | Mnm::MPSADBW
        | Mnm::PCMPESTRI
        | Mnm::PCMPESTRM
        | Mnm::PCMPISTRM
        | Mnm::PCMPISTRI => ot_chk(
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
        Mnm::VPINSRB => ot_chk(
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
        Mnm::VPINSRQ => ot_chk(
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
        Mnm::VPINSRD => ot_chk(
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
        Mnm::VPEXTRB => ot_chk(
            ins,
            &[
                (&[M8, R32, R64, M32, M64], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VPEXTRD | Mnm::VEXTRACTPS => ot_chk(
            ins,
            &[
                (&[R32, M32], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VPEXTRQ => ot_chk(
            ins,
            &[
                (&[R64, M64], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VPEXTRW => ot_chk(
            ins,
            &[
                (&[M16], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VROUNDSS | Mnm::VINSERTPS => ot_chk(
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
        Mnm::VROUNDSD => ot_chk(
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
        Mnm::VROUNDPS | Mnm::VROUNDPD => ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[(XMM, YMM), (XMM, M256), (YMM, M128), (YMM, XMM)],
            &[],
        ),
        Mnm::VMOVAPS | Mnm::VMOVAPD | Mnm::VMOVUPS | Mnm::VMOVUPD | Mnm::VMOVDQA => ot_chk(
            ins,
            &[
                (&[XMM, YMM, M128, M256], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
            ],
            &[(XMM, M256), (XMM, YMM), (YMM, XMM), (YMM, M128), (MA, MA)],
            &[],
        ),
        Mnm::VMOVMSKPD => avx_ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VMOVSD => avx_ot_chk(
            ins,
            &[
                (&[XMM, M64], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
                (&[XMM], Optional::Optional),
            ],
            &[(MA, MA, XMM), (XMM, MA, XMM), (MA, XMM, XMM)],
            &[],
        ),
        Mnm::VMOVSS => avx_ot_chk(
            ins,
            &[
                (&[XMM, M32], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
                (&[XMM], Optional::Optional),
            ],
            &[(MA, MA, XMM), (XMM, MA, XMM), (MA, XMM, XMM)],
            &[],
        ),
        Mnm::VPMULDQ | Mnm::VPMULLD => avx_ot_chk(
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
        Mnm::VMOVLPS | Mnm::VMOVLPD | Mnm::VMOVHPS | Mnm::VMOVHPD => avx_ot_chk(
            ins,
            &[
                (&[XMM, M64], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[M64], Optional::Optional),
            ],
            &[(MA, XMM, MA)],
            &[],
        ),
        Mnm::VLDDQU | Mnm::VMOVNTDQA => ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[M128, M256], Optional::Needed),
            ],
            &[(XMM, M256), (YMM, M128)],
            &[],
        ),
        Mnm::VPHMINPOSUW => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::VMOVDDUP => ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M64, M256], Optional::Needed),
            ],
            &[(XMM, YMM), (XMM, M256), (YMM, XMM), (YMM, M64)],
            &[],
        ),
        Mnm::VMOVSLDUP
        | Mnm::VPTEST
        | Mnm::VMOVSHDUP
        | Mnm::VRCPPS
        | Mnm::VSQRTPS
        | Mnm::VRSQRTPS
        | Mnm::VSQRTPD => ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
            ],
            &[(XMM, M256), (XMM, YMM), (YMM, XMM), (YMM, M128)],
            &[],
        ),
        Mnm::VADDSD
        | Mnm::VSUBSD
        | Mnm::VMULSD
        | Mnm::VDIVSD
        | Mnm::VSQRTSD
        | Mnm::VMINSD
        | Mnm::VMAXSD => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VADDSS
        | Mnm::VSUBSS
        | Mnm::VMULSS
        | Mnm::VDIVSS
        | Mnm::VRCPSS
        | Mnm::VSQRTSS
        | Mnm::VRSQRTSS
        | Mnm::VMINSS
        | Mnm::VMAXSS => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VADDPD
        | Mnm::VSUBPD
        | Mnm::VDIVPD
        | Mnm::VMULPD
        | Mnm::VMINPD
        | Mnm::VMAXPD
        | Mnm::VORPD
        | Mnm::VANDNPD
        | Mnm::VANDPD
        | Mnm::VXORPD
        | Mnm::VADDPS
        | Mnm::VSUBPS
        | Mnm::VDIVPS
        | Mnm::VMULPS
        | Mnm::VMINPS
        | Mnm::VMAXPS
        | Mnm::VORPS
        | Mnm::VANDNPS
        | Mnm::VANDPS
        | Mnm::VUNPCKLPS
        | Mnm::VUNPCKHPS
        | Mnm::VADDSUBPS
        | Mnm::VADDSUBPD
        | Mnm::VHADDPS
        | Mnm::VHADDPD
        | Mnm::VHSUBPS
        | Mnm::VHSUBPD
        | Mnm::VPMAXSB
        | Mnm::VPMAXSD
        | Mnm::VPMINSB
        | Mnm::VPMINSD
        | Mnm::VPMAXUW
        | Mnm::VPMAXUB
        | Mnm::VPMINUW
        | Mnm::VPMINUB
        | Mnm::VPCMPEQQ
        | Mnm::VPCMPGTQ
        | Mnm::VPACKUSDW
        | Mnm::VXORPS => avx_ot_chk(
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
        Mnm::VCOMISD | Mnm::VUCOMISD => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M64], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::VCOMISS | Mnm::VUCOMISS => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M32], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::VMOVHLPS | Mnm::VMOVLHPS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VPCMPESTRI | Mnm::VPCMPESTRM | Mnm::VPCMPISTRI | Mnm::VPCMPISTRM => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VCMPSS | Mnm::VCMPSD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VBLENDVPS | Mnm::VBLENDVPD | Mnm::VPBLENDVB => avx_ot_chk(
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

        Mnm::VBLENDPS
        | Mnm::VBLENDPD
        | Mnm::VPBLENDW
        | Mnm::VMPSADBW
        | Mnm::VDPPS
        | Mnm::VDPPD
        | Mnm::VCMPPS
        | Mnm::VCMPPD
        | Mnm::VSHUFPS => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
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
        Mnm::VPOR
        | Mnm::VPAND
        | Mnm::VPXOR
        | Mnm::VPADDB
        | Mnm::VPADDW
        | Mnm::VPADDD
        | Mnm::VPADDQ
        | Mnm::VPSUBB
        | Mnm::VPSUBD
        | Mnm::VPSUBQ
        | Mnm::VPSUBW
        | Mnm::VPANDN
        | Mnm::VPSUBSW
        | Mnm::VPSUBSB
        | Mnm::VPADDSB
        | Mnm::VPADDSW
        | Mnm::VPMULLW
        | Mnm::VPSUBUSB
        | Mnm::VPSUBUSW
        | Mnm::VPADDUSB
        | Mnm::VPADDUSW
        | Mnm::VPMADDWD
        | Mnm::VPCMPEQB
        | Mnm::VPCMPEQW
        | Mnm::VPCMPEQD
        | Mnm::VPCMPGTB
        | Mnm::VPCMPGTW
        | Mnm::VPCMPGTD
        | Mnm::VPACKUSWB
        | Mnm::VPACKSSWB
        | Mnm::VPACKSSDW
        | Mnm::VPUNPCKLBW
        | Mnm::VPUNPCKHBW
        | Mnm::VPUNPCKLWD
        | Mnm::VPUNPCKHWD
        | Mnm::VPUNPCKLDQ
        | Mnm::VPUNPCKHDQ
        | Mnm::VPMULHW => avx_ot_chk(
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
        Mnm::VPSLLW
        | Mnm::VPSLLD
        | Mnm::VPSLLQ
        | Mnm::VPSRLW
        | Mnm::VPSRLD
        | Mnm::VPSRLQ
        | Mnm::VPSRAD
        | Mnm::VPSRAW => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256, I8], Optional::Needed),
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
        Mnm::VMOVD => ot_chk(
            ins,
            &[
                (&[XMM, R32, M32], Optional::Needed),
                (&[XMM, R32, M32], Optional::Needed),
            ],
            &[(M32, M32), (R32, R32), (XMM, XMM)],
            &[],
        ),
        Mnm::VMOVQ => ot_chk(
            ins,
            &[
                (&[XMM, R64, M64], Optional::Needed),
                (&[XMM, R64, M64], Optional::Needed),
            ],
            &[(M64, M64), (R64, R64), (XMM, XMM)],
            &[],
        ),
        // part2a
        Mnm::VZEROALL | Mnm::VZEROUPPER => ot_chk(ins, &[], &[], &[]),
        Mnm::PAVGB | Mnm::PAVGW => ot_chk(
            ins,
            &[
                (&[XMM, MMX], Optional::Needed),
                (&[XMM, MMX], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VPAVGB | Mnm::VPAVGW | Mnm::VPHADDW | Mnm::VPHADDD | Mnm::VPHSUBW | Mnm::VPHSUBD => {
            avx_ot_chk(
                ins,
                &[
                    (&[XMM, YMM], Optional::Needed),
                    (&[XMM, YMM], Optional::Needed),
                    (&[XMM, YMM, M128, M256], Optional::Needed),
                ],
                &[],
                &[],
            )
        }
        Mnm::VBROADCASTF128 => avx_ot_chk_wthout(
            ins,
            &[(&[YMM], Optional::Needed), (&[M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::VBROADCASTSD => avx_ot_chk_wthout(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VBROADCASTSS => avx_ot_chk_wthout(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VEXTRACTF128 => avx_ot_chk_wthout(
            ins,
            &[
                (&[XMM, M128], Optional::Needed),
                (&[YMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VINSERTF128 => avx_ot_chk_wthout(
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
        Mnm::VPALIGNR => avx_ot_chk(
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
        Mnm::STMXCSR | Mnm::VSTMXCSR | Mnm::LDMXCSR | Mnm::VLDMXCSR => {
            ot_chk(ins, &[(&[M32], Optional::Needed)], &[], &[])
        }
        Mnm::VMOVMSKPS => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VPERMILPD | Mnm::VPERMILPS => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
                (&[XMM, YMM, M256, M128, I8], Optional::Needed),
            ],
            &[(XMM, MA, MA), (YMM, MA, MA)],
            &[],
        ),
        Mnm::VPCLMULQDQ => avx_ot_chk(
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
        Mnm::PCLMULQDQ => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, M128], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VPERM2F128 | Mnm::VPERM2I128 => avx_ot_chk(
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
        Mnm::VPMAXSW
        | Mnm::VPMINSW
        | Mnm::VPSIGNB
        | Mnm::VPSIGNW
        | Mnm::VPSIGND
        | Mnm::VPMULUDQ
        | Mnm::VPMULHUW
        | Mnm::VPMULHRSW => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VPSRLDQ => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[I8], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VPINSRW => avx_ot_chk(
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
        Mnm::VPMAXUD => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, M128, YMM, M256], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::PMAXSW | Mnm::PMINSW | Mnm::PMULHUW => ot_chk(
            ins,
            &[
                (&[XMM, MMX], Optional::Needed),
                (&[XMM, MMX], Optional::Needed),
            ],
            &[(XMM, MMX), (MMX, XMM)],
            &[],
        ),
        Mnm::PMAXUD => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::PINSRW => ot_chk(
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
        Mnm::VFMADD132PD
        | Mnm::VFMADD213PD
        | Mnm::VFMADD231PD
        | Mnm::VFMSUB132PD
        | Mnm::VFMSUB213PD
        | Mnm::VFMSUB231PD
        | Mnm::VFMADD132PS
        | Mnm::VFMADD213PS
        | Mnm::VFMADD231PS
        | Mnm::VFMSUB132PS
        | Mnm::VFMSUB213PS
        | Mnm::VFMSUB231PS => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VFMADD132SS
        | Mnm::VFMADD213SS
        | Mnm::VFMADD231SS
        | Mnm::VFMSUB132SS
        | Mnm::VFMSUB213SS
        | Mnm::VFMSUB231SS => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VFMADD132SD
        | Mnm::VFMADD213SD
        | Mnm::VFMADD231SD
        | Mnm::VFMSUB132SD
        | Mnm::VFMSUB213SD
        | Mnm::VFMSUB231SD => avx_ot_chk(
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
        Mnm::VFNMADD132PD
        | Mnm::VFNMADD213PD
        | Mnm::VFNMADD231PD
        | Mnm::VFNMSUB132PD
        | Mnm::VFNMSUB213PD
        | Mnm::VFNMSUB231PD
        | Mnm::VFNMADD132PS
        | Mnm::VFNMADD213PS
        | Mnm::VFNMADD231PS
        | Mnm::VFNMSUB132PS
        | Mnm::VFNMSUB213PS
        | Mnm::VFNMSUB231PS => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VFNMADD132SS
        | Mnm::VFNMADD213SS
        | Mnm::VFNMADD231SS
        | Mnm::VFNMSUB132SS
        | Mnm::VFNMSUB213SS
        | Mnm::VFNMSUB231SS => avx_ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VFNMADD132SD
        | Mnm::VFNMADD213SD
        | Mnm::VFNMADD231SD
        | Mnm::VFNMSUB132SD
        | Mnm::VFNMSUB213SD
        | Mnm::VFNMSUB231SD => avx_ot_chk(
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
        Mnm::VFMADDSUB132PS
        | Mnm::VFMADDSUB213PS
        | Mnm::VFMADDSUB231PS
        | Mnm::VFMSUBADD132PS
        | Mnm::VFMSUBADD213PS
        | Mnm::VFMSUBADD231PS
        | Mnm::VFMADDSUB132PD
        | Mnm::VFMADDSUB213PD
        | Mnm::VFMADDSUB231PD
        | Mnm::VFMSUBADD132PD
        | Mnm::VFMSUBADD213PD
        | Mnm::VFMSUBADD231PD => avx_ot_chk(
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
        Mnm::AESDEC | Mnm::AESENC | Mnm::AESIMC | Mnm::VAESIMC | Mnm::AESDECLAST => avx_ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::VAESDEC | Mnm::VAESENC | Mnm::VAESDECLAST => avx_ot_chk(
            ins,
            &[
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::AESKEYGENASSIST | Mnm::VAESKEYGENASSIST => avx_ot_chk(
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
        Mnm::CVTPD2PI => ot_chk(
            ins,
            &[(&[MMX], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::CVTSS2SI => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::CVTTPD2PI => ot_chk(
            ins,
            &[(&[MMX], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::CVTPS2PI | Mnm::CVTTPS2PI => ot_chk(
            ins,
            &[(&[MMX], Optional::Needed), (&[XMM, M64], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::CVTPI2PS | Mnm::CVTPI2PD => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[MMX, M64], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::CVTSI2SS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[R32, R64, M32, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::CVTPD2DQ
        | Mnm::CVTPS2DQ
        | Mnm::CVTTPS2DQ
        | Mnm::CVTTPD2DQ
        | Mnm::CVTDQ2PS
        | Mnm::CVTPD2PS => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::CVTSS2SD => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M32], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::CVTDQ2PD | Mnm::CVTPS2PD | Mnm::CVTSD2SS => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M64], Optional::Needed)],
            &[],
            &[],
        ),
        Mnm::CVTTSS2SI => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::CVTSD2SI | Mnm::CVTTSD2SI => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::CVTSI2SD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM, R32, R64, M32, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        // cvt-part2
        Mnm::VCVTSI2SD | Mnm::VCVTSI2SS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[R64, R32, M32, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VCVTSS2SD => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VCVTDQ2PS | Mnm::VCVTTPD2DQ | Mnm::VCVTTPS2DQ => avx_ot_chk(
            ins,
            &[
                (&[YMM, XMM], Optional::Needed),
                (&[XMM, YMM, M128, M256], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VCVTDQ2PD => ot_chk(
            ins,
            &[
                (&[YMM, XMM], Optional::Needed),
                (&[XMM, M64, M128], Optional::Needed),
            ],
            &[(YMM, M64), (XMM, M128)],
            &[],
        ),
        Mnm::VCVTSD2SS => ot_chk(
            ins,
            &[
                (&[XMM], Optional::Needed),
                (&[XMM], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VCVTTSD2SI => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VCVTSS2SI | Mnm::VCVTTSS2SI => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, M32], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VCVTSD2SI => ot_chk(
            ins,
            &[
                (&[R32, R64], Optional::Needed),
                (&[XMM, M64], Optional::Needed),
            ],
            &[],
            &[],
        ),
        Mnm::VCVTPD2DQ | Mnm::VCVTPD2PS | Mnm::VCVTPS2DQ | Mnm::VCVTPS2PD => ot_chk(
            ins,
            &[(&[XMM], Optional::Needed), (&[XMM, M128], Optional::Needed)],
            &[],
            &[],
        ),

        _ => Some(RASMError::no_tip(
            Some(ins.line),
            Some("Tried to use currently unsupported instruction."),
        )),
    }
}

// Utils

#[derive(PartialEq)]
enum Optional {
    Needed,
    Optional,
}

fn avx_ot_chk_wthout(
    ins: &Instruction,
    ops: &[(&[AType], Optional)],
    forb: &[(AType, AType, AType)],
    addt: &[Mnm],
) -> Option<RASMError> {
    if let Some(err) = addt_chk(ins, addt) {
        return Some(err);
    }
    if ops.is_empty() && !ins.oprs.is_empty() {
        return Some(RASMError::no_tip(
            Some(ins.line),
            Some("Instruction doesn't accept any operand, but you tried to use one anyways"),
        ));
    }
    for (idx, allowed) in ops.iter().enumerate() {
        if let Some(op) = ins.get_opr(idx) {
            if let Some(err) = type_check(op, allowed.0, idx) {
                return Some(err);
            }
        } else {
            if allowed.1 == Optional::Needed {
                return Some(RASMError::no_tip(
                    Some(ins.line),
                    Some(format!("Needed operand not found at index {}", idx)),
                ));
            } else {
                break;
            }
        }
    }
    if ops.len() == 2 {
        if let Some(err) = size_chk(ins) {
            return Some(err);
        }
    }
    if let Some(err) = avx_forb_chk(ins, forb) {
        return Some(err);
    }
    None
}
fn avx_ot_chk(
    ins: &Instruction,
    ops: &[(&[AType], Optional)],
    forb: &[(AType, AType, AType)],
    addt: &[Mnm],
) -> Option<RASMError> {
    if let Some(err) = addt_chk(ins, addt) {
        return Some(err);
    }
    if ops.is_empty() && !ins.oprs.is_empty() {
        return Some(RASMError::no_tip(
            Some(ins.line),
            Some("Instruction doesn't accept any operand, but you tried to use one anyways"),
        ));
    }
    for (idx, allowed) in ops.iter().enumerate() {
        if let Some(op) = ins.get_opr(idx) {
            if let Some(err) = type_check(op, allowed.0, idx) {
                return Some(err);
            }
        } else {
            if allowed.1 == Optional::Needed {
                return Some(RASMError::no_tip(
                    Some(ins.line),
                    Some(format!("Needed operand not found at index {}", idx)),
                ));
            } else {
                break;
            }
        }
    }
    if ops.len() == 2 {
        if let Some(err) = avx_size_chk(ins) {
            return Some(err);
        }
    }
    if let Some(err) = avx_forb_chk(ins, forb) {
        return Some(err);
    }
    None
}

fn avx_forb_chk(ins: &Instruction, forb: &[(AType, AType, AType)]) -> Option<RASMError> {
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
            return Some(RASMError::no_tip(
                Some(ins.line),
                Some(format!(
                    "Destination, AVX Source and Source operand have forbidden combination: ({}, {})",
                    f.0.to_string(),
                    f.1.to_string()
                )),
            ));
        }
    }
    None
}

fn ot_chk(
    ins: &Instruction,
    ops: &[(&[AType], Optional)],
    forb: &[(AType, AType)],
    addt: &[Mnm],
) -> Option<RASMError> {
    if let Some(err) = addt_chk(ins, addt) {
        return Some(err);
    }
    if ops.is_empty() && !(ins.oprs == [None, None, None, None, None]) {
        return Some(RASMError::no_tip(
            Some(ins.line),
            Some("Instruction doesn't accept any operand, but you tried to use one anyways"),
        ));
    }
    for (idx, allowed) in ops.iter().enumerate() {
        if let Some(op) = ins.get_opr(idx) {
            if let Some(err) = type_check(op, allowed.0, idx) {
                return Some(err);
            }
        } else {
            if allowed.1 == Optional::Needed {
                return Some(RASMError::no_tip(
                    Some(ins.line),
                    Some(format!("Needed operand not found at index {}", idx)),
                ));
            } else {
                break;
            }
        }
    }
    if ops.len() == 2 {
        if let Some(err) = size_chk(ins) {
            let mut b = false;
            for o in ops {
                for o in o.0 {
                    if let AType::ExtendedRegister(_) = o {
                        b = true;
                        break;
                    }
                }
            }
            if !b {
                return Some(err);
            }
        }
    }
    if let Some(err) = forb_chk(ins, forb) {
        return Some(err);
    }
    None
}

fn forb_chk(ins: &Instruction, forb: &[(AType, AType)]) -> Option<RASMError> {
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
            return Some(RASMError::no_tip(
                Some(ins.line),
                Some(format!(
                    "Destination and Source operand have forbidden combination: ({}, {})",
                    f.0.to_string(),
                    f.1.to_string()
                )),
            ));
        }
    }
    None
}

fn type_check(operand: &Operand, accepted: &[AType], idx: usize) -> Option<RASMError> {
    if let Some(m) = operand.get_mem() {
        if m.addrsize() == Some(Size::Word) {
            return Some(RASMError::no_tip(
                None,
                Some("You cannot address with 16-bit registers; consider using 32-bit/64-bit (depending on bits).")
            ));
        }
    }
    if find(accepted, operand.atype()) || find_ext(accepted, operand.ext_atype()) {
        None
    } else {
        let err = RASMError::with_tip(
            None,
            Some(format!(
                "{} operand of type {} doesn't match any of expected types: {}",
                match idx {
                    0 => "Destination".to_string(),
                    1 => "Source".to_string(),
                    _ => idx.to_string(),
                },
                operand.atype().to_string(),
                atype_arr_string(accepted)
            )),
            Some(format!(
                "Consider changing {} operand to expected type or removing instruction",
                match idx {
                    0 => "destination".to_string(),
                    1 => "source".to_string(),
                    _ => idx.to_string(),
                }
            )),
        );

        if let Operand::Imm(imm) = operand {
            if accepted.contains(&AType::Immediate(imm.size())) {
                return None;
            }
        }
        Some(err)
    }
}
fn avx_size_chk(ins: &Instruction) -> Option<RASMError> {
    let dst = ins.dst().unwrap();
    let src = ins.src().unwrap();

    if let Operand::CtrReg(_) = dst {
        return None;
    }
    if let Operand::CtrReg(_) = src {
        return None;
    }
    // should work (i hope so)
    match (dst.atype(), src.atype()) {
        (AType::Register(_, s0) | AType::Memory(s0) | AType::SMemory(s0), AType::Immediate(s1)) => {
            if s1 <= s0 {
                None
            } else {
                if !ins.mnem.allows_diff_size(Some(s0), Some(s1)) {
                    Some(RASMError::with_tip(
                        Some(ins.line),
                        Some("Tried to use immediate that is too large for destination operand"),
                        Some(format!("Consider changing immediate to fit inside {s0}",)),
                    ))
                } else {
                    None
                }
            }
        }
        (AType::Memory(s0) | AType::SMemory(s0), AType::Memory(s1) | AType::SMemory(s1)) => {
            if s1 == s0 {
                None
            } else {
                if !ins.mnem.allows_diff_size(Some(s0), Some(s1)) {
                    Some(RASMError::with_tip(
                        Some(ins.line),
                        Some("Tried to use operand that cannot be used for destination operand"),
                        Some(format!("Consider changing operand to be {s0}",)),
                    ))
                } else {
                    None
                }
            }
        }
        (AType::Register(_, s0), AType::Register(_, s1)) => {
            if let Some(ssrc) = ins.src2() {
                if s1 == s0 && ssrc.size() == s0 {
                    None
                } else {
                    Some(RASMError::with_tip(
                        Some(ins.line),
                        Some("Tried to use operand that cannot be used for destination operand"),
                        Some(format!("Consider changing operand to be {s0}",)),
                    ))
                }
            } else if s1 == s0 {
                None
            } else {
                if !ins.mnem.allows_diff_size(Some(s0), Some(s1)) {
                    Some(RASMError::with_tip(
                        Some(ins.line),
                        Some("Tried to use operand that cannot be used for destination operand"),
                        Some(format!("Consider changing operand to be {s0}",)),
                    ))
                } else {
                    None
                }
            }
        }

        _ => None,
    }
}
fn size_chk(ins: &Instruction) -> Option<RASMError> {
    let dst = ins.dst().unwrap();
    let src = ins.src().unwrap();

    if let Operand::CtrReg(_) = dst {
        return None;
    }
    if let Operand::CtrReg(_) = src {
        return None;
    }
    // should work (i hope so)
    match (dst.atype(), src.atype()) {
        (AType::Register(_, s0) | AType::Memory(s0) | AType::SMemory(s0), AType::Immediate(s1)) => {
            if s1 <= s0 {
                None
            } else {
                if !ins.mnem.allows_diff_size(Some(s0), Some(s1)) {
                    Some(RASMError::with_tip(
                        Some(ins.line),
                        Some("Tried to use immediate that is too large for destination operand"),
                        Some(format!("Consider changing immediate to fit inside {s0}",)),
                    ))
                } else {
                    None
                }
            }
        }
        (AType::Memory(s0) | AType::SMemory(s0), AType::Memory(s1) | AType::SMemory(s1)) => {
            if s1 == s0 {
                None
            } else {
                if !ins.mnem.allows_diff_size(Some(s0), Some(s1)) {
                    Some(RASMError::with_tip(
                        Some(ins.line),
                        Some("Tried to use operand that cannot be used for destination operand"),
                        Some(format!("Consider changing operand to fit inside {s0}",)),
                    ))
                } else {
                    None
                }
            }
        }
        (AType::Register(g0, s0), AType::Register(g1, s1)) => {
            if s1 == s0
                || ((g0 == RPurpose::Dbg
                    || g0 == RPurpose::Ctrl
                    || g0 == RPurpose::Sgmnt
                    || g0 == RPurpose::Mmx
                    || g0 == RPurpose::F128
                    || g0 == RPurpose::F256)
                    || (g1 == RPurpose::Dbg
                        || g1 == RPurpose::Ctrl
                        || g1 == RPurpose::Sgmnt
                        || g1 == RPurpose::Mmx
                        || g1 == RPurpose::F128
                        || g1 == RPurpose::F256))
            {
                None
            } else {
                if !ins.mnem.allows_diff_size(Some(s0), Some(s1)) {
                    Some(RASMError::with_tip(
                        Some(ins.line),
                        Some("Tried to use operand that cannot be used for destination operand"),
                        Some(format!("Consider changing operand to be {s0}",)),
                    ))
                } else {
                    None
                }
            }
        }

        _ => None,
    }
}

fn addt_chk(ins: &Instruction, accpt_addt: &[Mnm]) -> Option<RASMError> {
    if let Some(addt) = &ins.addt {
        if !find_bool(accpt_addt, addt) {
            return Some(RASMError::no_tip(
                Some(ins.line),
                Some(format!(
                    "Use of forbidden additional mnemonic: {}",
                    addt.to_string()
                )),
            ));
        }
    }
    None
}

fn find_bool(addts: &[Mnm], searched: &Mnm) -> bool {
    for addt in addts {
        if searched == addt {
            return true;
        }
    }
    false
}

fn find_ext(items: &[AType], searched: AType) -> bool {
    let (size, regprp, reg) = match searched {
        AType::Register(prp, size) => (size, Some(prp), None),
        AType::Immediate(size) => (size, None, None),
        AType::SMemory(size) | AType::Memory(size) => (size, None, None),
        AType::Symbol => (Size::Any, None, None),
        AType::ExtendedRegister(r) => (r.size(), Some(r.purpose()), Some(r)),
    };
    for i in items {
        let (isize, iregprp, ireg) = match i {
            AType::Register(prp, size) => (*size, Some(*prp), None),
            AType::Immediate(size) => (*size, None, None),
            AType::SMemory(size) | AType::Memory(size) => (*size, None, None),
            AType::Symbol => (Size::Any, None, None),
            AType::ExtendedRegister(r) => (r.size(), Some(r.purpose()), Some(*r)),
        };
        if let (Some(ireg), Some(reg)) = (ireg, reg) {
            if ireg == reg {
                return true;
            }
        } else if isize == size && regprp == iregprp {
            return true;
        }
    }
    false
}
fn find(items: &[AType], searched: AType) -> bool {
    let (size, regprp) = match searched {
        AType::Register(prp, size) => (size, Some(prp)),
        AType::Immediate(size) => (size, None),
        AType::SMemory(size) | AType::Memory(size) => (size, None),
        AType::Symbol => (Size::Any, None),
        _ => (Size::Unknown, None),
    };
    for i in items {
        let (isize, iregprp) = match i {
            AType::Register(prp, size) => (size, Some(prp)),
            AType::Immediate(size) => (size, None),
            AType::SMemory(size) | AType::Memory(size) => (size, None),
            AType::Symbol => (&Size::Any, None),
            _ => (&Size::Unknown, None),
        };
        if isize == &size && regprp.as_ref() == iregprp {
            return true;
        }
    }
    false
}
