// pasm - src/core/api.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

const _RESERVED_FLAG_0: u8 = 0x0;
const _RESERVED_FLAG_1: u8 = 0x1;
const _RESERVED_FLAG_2: u8 = 0x2;

pub const CAN_H66O: u8 = 0x3; // can use 0x66 override
pub const IMM_LEBE: u8 = 0x4; // immediate must be formatted as little endian or big endian; 0 = le, 1 = be
pub const CAN_SEGM: u8 = 0x5; // can use segment override
pub const USE_MODRM: u8 = 0x6; // can use modrm
const _RESERVED_FLAG_7: u8 = 0x7;
const _RESERVED_FLAG_8: u8 = 0x8;
pub const IMM: u8 = 0x9; // immediate at index (second byte of addt is index, first one is size)
pub const SET_MODRM: u8 = 0xA; // MODRM.mod is set to byte specified in addt2
pub const STRICT_PFX: u8 = 0xB; // makes all prefixes exclusive (e.g. if EVEX isn't set, then it cannot be generated)
pub const FIXED_SIZE: u8 = 0xC;

use std::iter::Iterator;

use crate::{
    core::{apx, disp, evex, modrm, rex, sib, vex},
    shr::{
        booltable::BoolTable16,
        instruction::{Instruction, Operand},
        mem::Mem,
        mnemonic::Mnemonic,
        reg::Register,
        reloc::RelType as RelocationType,
        reloc::Relocation,
        size::Size,
        stackvec::StackVec,
    },
};

#[repr(C)]
pub struct VexDetails {
    pp: u8,
    map_select: u8,
    vex_we: bool,
    vlength: MegaBool,
}

#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct MegaBool {
    data: u8,
}

#[repr(transparent)]
pub struct OperandOrder {
    ord: u8,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum OpOrd {
    MODRM_RM = 0b00,
    MODRM_REG = 0b01,
    VEX_VVVV = 0b10, // vex source (second source operand)
    TSRC = 0b11,     // third source
}

pub const EVEX_VVVV: u8 = OpOrd::VEX_VVVV as u8;

pub const PREFIX_NONE: u8 = 0b000;
pub const PREFIX_REX: u8 = 0b001;
pub const PREFIX_VEX: u8 = 0b010;
pub const PREFIX_EVEX: u8 = 0b011;
pub const PREFIX_APX: u8 = 0b100;

// size = 16B
#[repr(C)]
pub struct GenAPI {
    pub flags: BoolTable16,
    // TODO: patch, so (E)VEX vlength can be where prefix is and not in addt
    // if PREFIX_(E)VEX is set, the byte is 0bX_YYYYY_ZZ where:
    // X = (E)VEX.w/e,
    // YYYYY = map_select
    // ZZ = pp
    // otherwise normal prefix like 0xF2/0xF3
    prefix: u16,

    // depending on flags:
    // - IMM - immediate + metadata,
    // - (E)VEX_PFX - first byte (last 2 bits) is reserved for vlength (and is cleared during .assemble()
    // otherwise unused
    addt: u16,
    opcode: [u8; 4],
    // layout:
    //  0bPPPR_0LLL:
    //   PPP: forced prefix (fpfx):
    //      0b000 - None
    //      0b001 - REX
    //      0b010 - VEX
    //      0b011 - EVEX
    //      0b100 - APX (conditional CMP/TEST, otherwise guess yourself)
    //      0b... - reserved
    //   R: reserved
    //   LLL: opcode length
    meta_0: u8,
    _reserved: [u8; 2],

    // less essential - can be used with other context depending on flags

    // can be used with other context if USE_MODRM flag is NOT set
    modrm_ovr: ModrmTuple,

    // can be used with other context if USE_MODRM flag is NOT set (because why would you need it?)
    ord: OperandOrder,

    // - FIXED_SIZE - reserved for size
    // - SET_MODRM  - reserved for modrm.mod
    addt2: u8,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ModrmTuple {
    data: u8, // 0bXX_YYY_ZZZ : X1 = reg is Some, X2 = rm is Some, YYY = reg, ZZZ = rm
}

pub enum AssembleResult {
    NoLargeImm(StackVec<u8, 16>),
    WLargeImm(Vec<u8>),
}

impl GenAPI {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            opcode: [0; 4],
            meta_0: 0,
            _reserved: [0; 2],
            prefix: 0,
            flags: BoolTable16::new().setc(CAN_H66O, true).setc(CAN_SEGM, true),
            ord: OperandOrder::new(&[
                OpOrd::MODRM_RM,
                OpOrd::MODRM_REG,
                OpOrd::VEX_VVVV,
                OpOrd::TSRC,
            ])
            .unwrap(),
            modrm_ovr: ModrmTuple::new(None),
            addt: 0,
            addt2: 0,
        }
    }
    #[inline(always)]
    pub const fn set_fpfx(&mut self, v: u8) {
        self.meta_0 &= !0b1110_0000;
        self.meta_0 |= v << 5;
    }
    #[inline(always)]
    pub const fn get_fpfx(&self) -> u8 {
        (self.meta_0 & 0b1110_0000) >> 5
    }
    #[inline(always)]
    pub const fn opcode_prefix(mut self, pfx: u8) -> Self {
        self.prefix = pfx as u16;
        self
    }
    #[inline(always)]
    pub fn opcode_len(&self) -> usize {
        (self.meta_0 & 0b111) as usize
    }
    #[inline(always)]
    pub const fn opcode(mut self, opc: &[u8]) -> Self {
        let mut idx = 0;
        while idx < opc.len() {
            self.opcode[idx] = opc[idx];
            idx += 1;
        }
        self.meta_0 |= idx as u8 & 0b111;
        self
    }
    #[inline(always)]
    pub fn modrm(mut self, modrm: bool, reg: Option<u8>) -> Self {
        self.flags.set(USE_MODRM, modrm);
        self.modrm_ovr = ModrmTuple::new(reg);
        self
    }
    #[inline(always)]
    pub const fn fixed_size(mut self, sz: Size) -> Self {
        self.flags.set(FIXED_SIZE, true);
        self.addt2 |= sz as u8;
        self
    }
    #[inline(always)]
    pub const fn modrm_mod(mut self, mod_: u8) -> Self {
        self.flags.set(SET_MODRM, true);
        self.addt2 = mod_ & 0b11;
        self
    }
    #[inline(always)]
    pub const fn can_h66(mut self, h66: bool) -> Self {
        self.flags.set(CAN_H66O, h66);
        self
    }

    // prefix related things go there (setters only).
    #[inline(always)]
    pub const fn rex(mut self) -> Self {
        self.set_fpfx(PREFIX_REX);
        self
    }
    #[inline(always)]
    pub const fn evex(mut self, vex_details: VexDetails) -> Self {
        self.set_fpfx(PREFIX_EVEX);
        self.prefix = (vex_details.vex_we as u16) << 7
            | (vex_details.map_select as u16) << 2
            | pp(vex_details.pp) as u16;
        self.addt = ((vex_details.vlength.data as u16) << 0x08) | self.addt & 0x00FF;
        self
    }
    #[inline(always)]
    pub const fn vex(mut self, vex_details: VexDetails) -> Self {
        self.set_fpfx(PREFIX_VEX);
        self.prefix = {
            (vex_details.vex_we as u16) << 7
                | (map_select(vex_details.map_select) as u16) << 2
                | pp(vex_details.pp) as u16
        };
        self.addt = ((vex_details.vlength.data as u16) << 0x08) | self.addt & 0x00FF;
        self
    }

    // Immediate-related things go here
    #[inline(always)]
    pub const fn imm_atindex(mut self, idx: u16, size: u16) -> Self {
        self.flags.set(IMM, true);
        self.addt = ((size << 4) | idx & 0b1111) | self.addt & 0xFF00;
        self
    }
    #[inline(always)]
    pub const fn imm_is_be(mut self, bool: bool) -> Self {
        self.flags.set(IMM_LEBE, bool);
        self
    }

    #[inline(always)]
    pub const fn ord(mut self, ord: &[OpOrd]) -> Self {
        self.ord = OperandOrder::new(ord).expect("Failed to create operand order");
        self
    }
    #[inline(always)]
    pub const fn get_addt2(&self) -> u8 {
        self.addt2
    }
    #[inline(always)]
    pub fn get_size(&self) -> Option<Size> {
        if self.flags.at(FIXED_SIZE) {
            Some(unsafe { std::mem::transmute::<u8, Size>(self.addt2) })
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn get_opcode(&self) -> &[u8] {
        &self.opcode[0..self.opcode_len()]
    }
    // you can have max 2 relocations returned, because of variants like:
    // mov .deref @symbol, @other_symbol
    // (it is in fact a valid variant: first operand is mem, second is immediate)
    pub fn assemble<'a>(
        self,
        ins: &'a Instruction,
        bits: u8,
        default_rel: RelocationType,
    ) -> (AssembleResult, StackVec<Relocation<'a>, 2>) {
        let [modrm_rm, modrm_reg, _vex_vvvv] = self.get_ord_oprs(ins);

        let prefix_flag = self.get_fpfx();

        let mut rels = StackVec::<Relocation, 2>::new();

        let (ins_size, fx_size) = if let Some(sz) = self.get_size() {
            (sz, true)
        } else {
            (ins.size(), false)
        };

        let mut imm: Vec<u8> = Vec::new();
        let mut base: StackVec<u8, 16> = StackVec::new();

        if let Some(a) = gen_addt_pfx(ins) {
            base.push(a);
        }

        // this works (atleast i hope so?) for now
        let rex = if prefix_flag == PREFIX_REX && !self.flags.at(STRICT_PFX) && bits == 64 {
            rex::gen_rex(ins, self.ord.modrm_reg_is_dst()).unwrap_or(0)
        } else {
            0x00
        };

        let rexw = if bits == 64 {
            if rex != 0x00 {
                rex & 0x08 == 0x08
            } else {
                ins_size == Size::Qword || ins_size == Size::Any
            }
        } else {
            false
        };

        if let Some(segm) = gen_segm_pref(ins) {
            base.push(segm);
        }

        let not_std_prefix =
            prefix_flag == PREFIX_VEX || prefix_flag == PREFIX_EVEX || prefix_flag == PREFIX_APX;
        if fx_size {
            if let Some(size_ovr) = gen_sizeovr_fixed_size(ins_size, bits) {
                base.push(size_ovr);
            }
        } else if let Some(size_ovr) = gen_size_ovr(ins, &modrm_rm, ins_size, bits, rexw) {
            let h66 = size_ovr[0];
            let h67 = size_ovr[1];
            if h66.is_some() && !not_std_prefix && self.flags.at(CAN_H66O) && self.prefix != 0x66 {
                base.push(0x66);
            }
            if h67.is_some() {
                base.push(0x67);
            }
        }

        if !not_std_prefix && self.prefix != 0 {
            base.push(self.prefix.to_be_bytes()[1]);
        }

        // Prefixes
        match prefix_flag {
            PREFIX_REX => {
                if rex != 0x00 {
                    base.push(rex);
                }
            }
            PREFIX_VEX => {
                if ins.needs_apx_extension() {
                    for b in apx::apx(&self, ins, bits).into_iter() {
                        base.push(b);
                    }
                } else if ins.needs_evex() && !self.flags.at(STRICT_PFX) {
                    for b in evex::evex(&self, ins) {
                        base.push(b);
                    }
                } else {
                    for b in vex::vex(&self, ins).into_iter() {
                        base.push(b);
                    }
                }
            }
            PREFIX_EVEX => {
                if ins.needs_apx_extension() {
                    for b in apx::apx(&self, ins, bits).into_iter() {
                        base.push(b);
                    }
                } else {
                    for b in evex::evex(&self, ins) {
                        base.push(b);
                    }
                }
            }
            PREFIX_APX => {
                for b in apx::apx(&self, ins, bits).into_iter() {
                    base.push(b);
                }
            }
            _ => {}
        }

        // Opcode, max. 3-4B
        for b in self.get_opcode() {
            base.push(*b);
        }

        // ModRM, max. 1B
        if self.flags.at(USE_MODRM) {
            base.push(modrm::modrm(&modrm_rm, &modrm_reg, &self));
            // SIB, max. 1B
            // we cannot generate sib for bits == 16
            if bits != 16 {
                if let Some(sib) = sib::gen_sib_ins(&modrm_rm) {
                    base.push(sib);
                }
            }
            // DISP, max. 4B
            if let Some(disp) = disp::gen_disp_ins(&modrm_rm, bits) {
                for b in disp {
                    base.push(b);
                }
            } else {
                for (s, _) in ins.get_symbs().into_iter() {
                    if !s.is_deref() {
                        continue;
                    }

                    let reltype = s.reltype().unwrap_or(default_rel);
                    let addend = s.addend().unwrap_or_default();
                    rels.push(Relocation {
                        symbol: s.symbol,
                        offset: base.len(),
                        addend: addend - reltype.size() as i32,
                        shidx: 0,
                        reltype,
                    });
                    if bits != 16 {
                        base.push(0);
                        base.push(0);
                    }
                    base.push(0);
                    base.push(0);
                    break;
                }
            }
        }
        // Immediate, max. 8B or more (if string)
        if self.flags.at(IMM) {
            let size = ((self.addt & 0x00_F0) >> 4) as usize;
            let idx = (self.addt & 0x00_0F) as usize;

            match ins.get(idx) {
                Some(Operand::Imm(i)) => {
                    // if size is equal to zero, then it maps to `empty` mnemonic
                    if size == 0 {
                        let sz = i.get_as_usize();
                        imm.extend(vec![0; sz]);
                    } else {
                        let (imm, be) = if self.flags.at(IMM_LEBE) {
                            (&i.get_raw_be()[8 - i.get_real_size()..], true)
                        } else {
                            (&i.get_raw_le()[..i.get_real_size()], false)
                        };
                        let mut idx = 0;

                        let le = !be;

                        // if big endian add content before number like this (for fixed size):
                        // 0x0000_00FF
                        if be {
                            while idx < size.abs_diff(imm.len()) {
                                base.push(0x00);
                                idx += 1;
                            }
                        }
                        // add the actual number content
                        for b in imm {
                            if idx < size {
                                base.push(*b);
                                idx += 1;
                            } else {
                                break;
                            }
                        }
                        // if little endian add content after number like this (for fixed size):
                        // 0xFF00_0000
                        if le {
                            while idx < size {
                                base.push(0x00);
                                idx += 1;
                            }
                        }
                    }
                }
                Some(Operand::String(s)) => {
                    // we partition string, so we do not have situation like:
                    //  - add eax, "abc"
                    // and abc will add 3 byte INSTEAD OF 4 bytes.
                    let (sl_st, sl_en) = if size == 0 { (0, s.len()) } else { (0, size) };
                    let mut escape_char = false;
                    for b in &s.as_bytes()[sl_st..sl_en] {
                        if escape_char {
                            match *b {
                                b'n' => imm.push(b'\n'),
                                b't' => imm.push(b'\t'),
                                b'0' => imm.push(b'\0'),
                                b'r' => imm.push(b'\r'),
                                b'\"' => imm.push(b'\"'),
                                b'\'' => imm.push(b'\''),
                                _ => {
                                    imm.push(b'\\');
                                    imm.push(*b);
                                }
                            }
                            escape_char = false;
                        } else if *b == b'\\' {
                            escape_char = true;
                        } else {
                            imm.push(*b);
                        }
                    }
                }
                Some(Operand::Symbol(s)) => {
                    let reltype = s.reltype().unwrap_or(default_rel);
                    rels.push(Relocation {
                        symbol: s.symbol,
                        offset: base.len(),
                        addend: s.addend().unwrap_or_default() - reltype.size() as i32,
                        shidx: 0,
                        reltype,
                    });
                    if size == 0 {
                        for _ in 0..s.reltype().unwrap_or(default_rel).size() {
                            base.push(0);
                        }
                    } else {
                        for _ in 0..size {
                            base.push(0);
                        }
                    }
                }
                // rvrm
                Some(Operand::Register(r)) => {
                    let mut v = StackVec::<u8, 8>::new();
                    v.push((r.ebits()[1] as u8) << 7 | r.to_byte() << 4);
                    extend_imm(&mut v, size as u8);
                    for b in v.into_iter() {
                        base.push(b);
                    }
                }
                _ => {}
            }
        }

        if !imm.is_empty() {
            let mut vec = Vec::with_capacity(base.len() + imm.len());
            vec.extend(base.into_iter());
            vec.extend(imm);
            return (AssembleResult::WLargeImm(vec), rels);
        }
        (AssembleResult::NoLargeImm(base), rels)
    }
    #[inline(always)]
    pub const fn modrm_reg_is_dst(&self) -> bool {
        self.ord.modrm_reg_is_dst()
    }
    #[inline(always)]
    pub const fn get_modrm(&self) -> ModrmTuple {
        self.modrm_ovr
    }
    #[inline(always)]
    pub fn get_ord(&self) -> [OpOrd; 4] {
        self.ord.deserialize()
    }
    #[inline(always)]
    #[rustfmt::skip]
    pub fn get_ord_oprs<'a>(&'a self, ins: &'a Instruction) -> [Option<Operand<'a>>; 3] {
        use OpOrd::*;
        let ord = self.ord.deserialize();
        match &ord[..3] {
            //                                    MODRM.r/m   MODRM.reg   (E)VEX.vvvv
            [MODRM_REG, MODRM_RM , _        ] => [ins.src() , ins.dst() , ins.ssrc()],
            [MODRM_RM , MODRM_REG, _        ] => [ins.dst() , ins.src() , ins.ssrc()],
            [MODRM_REG, VEX_VVVV , MODRM_RM ] => [ins.ssrc(), ins.dst() , ins.src() ],
            [MODRM_RM , VEX_VVVV , MODRM_REG] => [ins.dst() , ins.ssrc(), ins.src() ],
            [VEX_VVVV , MODRM_REG, MODRM_RM ] => [ins.ssrc(), ins.src() , ins.dst() ],
            [VEX_VVVV , MODRM_REG, _        ] => [None      , ins.src() , ins.dst() ],
            [VEX_VVVV , MODRM_RM , _        ] => [ins.src() , None      , ins.dst() ],
            _                                 => [None      , None      , None      ],
        }
    }
    #[inline(always)]
    pub const fn strict_pfx(mut self) -> Self {
        self.flags.set(STRICT_PFX, true);
        self
    }
    // fails if (E)VEX flag is not set
    #[inline(always)]
    pub const fn get_pp(&self) -> Option<u8> {
        if self.get_fpfx() == PREFIX_VEX || self.get_fpfx() == PREFIX_EVEX {
            Some((self.prefix & 0b11) as u8)
        } else {
            None
        }
    }
    // fails if (E)VEX flag is not set
    #[inline(always)]
    pub const fn get_map_select(&self) -> Option<u8> {
        if self.get_fpfx() == PREFIX_VEX || self.get_fpfx() == PREFIX_EVEX {
            Some(((self.prefix & 0b0111_1100) >> 2) as u8)
        } else {
            None
        }
    }
    // fails if (E)VEX flag is not set
    #[inline(always)]
    pub const fn get_vex_we(&self) -> Option<bool> {
        if self.get_fpfx() == PREFIX_VEX || self.get_fpfx() == PREFIX_EVEX {
            Some(self.prefix & 0b1000_0000 == 0b1000_0000)
        } else {
            None
        }
    }
    // fails if (E)VEX flag is not set
    #[inline(always)]
    pub const fn get_vex_vlength(&self) -> Option<MegaBool> {
        if self.get_fpfx() == PREFIX_VEX || self.get_fpfx() == PREFIX_EVEX {
            Some(MegaBool::from_byte(((self.addt & 0xFF00) >> 8) as u8))
        } else {
            None
        }
    }

    // prefix layout for APX:
    //  0bAAAN_MMMM_MPPW_SSSS
    //  - AAA - APX Variant
    //  - N - ND
    //  - MMMMM - (E)VEX.map_select
    //  - PP - (E)VEX.pp
    //  - W - (E)VEX.w/e
    //  - SSSS - condition code
    pub fn apx(mut self, apx_variant: apx::APXVariant, apx_details: VexDetails, nd: bool) -> Self {
        self.prefix = 0;
        self.set_fpfx(PREFIX_APX);
        self.prefix |= (apx_variant as u16) << 13;
        self.prefix |= (nd as u16) << 12;
        self.prefix |= (apx_details.map_select as u16) << 7;
        self.prefix |= (pp(apx_details.pp) as u16) << 5;
        self.prefix |= (apx_details.vex_we as u16) << 4;
        self
    }
    pub fn apx_cccc(mut self, cccc: u8) -> Self {
        self.prefix |= (cccc as u16) & 0b1111;
        self
    }
    pub fn get_apx_cccc(&self) -> u8 {
        (self.prefix & 0b1111) as u8
    }
    pub fn get_apx_nd(&self) -> bool {
        self.prefix & 1 << 12 == 1 << 12
    }
    pub fn get_apx_eevex_map_select(&self) -> u8 {
        ((self.prefix & (0b11111 << 7)) >> 7) as u8
    }
    pub fn get_apx_eevex_pp(&self) -> u8 {
        ((self.prefix & (0b11 << 5)) >> 5) as u8
    }
    pub fn get_apx_eevex_vex_we(&self) -> bool {
        self.prefix & 1 << 4 == 1 << 4
    }
    pub fn get_apx_eevex_version(&self) -> Option<apx::APXVariant> {
        if self.get_fpfx() == PREFIX_EVEX {
            Some(apx::APXVariant::EvexExtension)
        } else if self.get_fpfx() == PREFIX_VEX {
            Some(apx::APXVariant::VexExtension)
        } else if self.get_fpfx() == PREFIX_APX {
            Some(unsafe { std::mem::transmute(((self.prefix & (0b111 << 13)) >> 13) as u8) })
        } else {
            None
        }
    }
}

// Assembling helper functions
fn gen_addt_pfx(ins: &Instruction) -> Option<u8> {
    use Mnemonic as Ins;
    if let Some(s) = ins.get_addt() {
        match s {
            Ins::LOCK => Some(0xF0),
            Ins::REPNE | Ins::REPNZ => Some(0xF2),
            Ins::REP | Ins::REPE | Ins::REPZ => Some(0xF3),
            _ => None,
        }
    } else {
        None
    }
}

fn gen_size_ovr(
    ins: &Instruction,
    dst: &Option<Operand>,
    sz: Size,
    bits: u8,
    rexw: bool,
) -> Option<[Option<u8>; 2]> {
    let mut arr = [None; 2];
    match bits {
        16 => {
            if let Size::Dword = sz {
                arr[0] = Some(0x66);
            }
        }
        32 => {
            if let Size::Word = sz {
                arr[0] = Some(0x66);
            }
        }
        64 => match sz {
            Size::Word => arr[0] = Some(0x66),
            Size::Qword => {
                if !(rexw || ins.mnemonic.defaults_to_64bit() || ins.uses_cr() || ins.uses_dr()) {
                    arr[0] = Some(0x66);
                }
            }
            _ => {}
        },
        _ => {}
    };
    if let Some(Operand::Mem(m)) = dst {
        match (m.addrsize(), bits) {
            (Size::Dword, 16) => arr[1] = Some(0x67),
            (Size::Dword, 64) => arr[1] = Some(0x67),
            _ => {}
        }
    }
    if arr[0].is_some() || arr[1].is_some() {
        Some(arr)
    } else {
        None
    }
}

// this has sense for instructions like lodsw, scasw, etc.
fn gen_sizeovr_fixed_size(sz: Size, bits: u8) -> Option<u8> {
    match (sz, bits) {
        (Size::Word, 32 | 64) => Some(0x66),
        (Size::Dword, 16) => Some(0x66),
        _ => None,
    }
}

fn gen_segm_pref(ins: &Instruction) -> Option<u8> {
    if let Some(mem) = ins.get_mem() {
        return gen_segm_pref_op(&mem);
    }
    None
}

fn extend_imm(imm: &mut StackVec<u8, 8>, size: u8) {
    let size = size as usize;
    while imm.len() < size {
        imm.push(0)
    }
}

const fn gen_segm_pref_op(mem: &Mem) -> Option<u8> {
    match mem.get_segment() {
        Some(Register::CS) => Some(0x2E),
        Some(Register::SS) => Some(0x36),
        Some(Register::DS) => Some(0x3E),
        Some(Register::ES) => Some(0x26),
        Some(Register::FS) => Some(0x64),
        Some(Register::GS) => Some(0x65),
        _ => None,
    }
}

// Implementations

impl Default for VexDetails {
    fn default() -> Self {
        Self::new()
    }
}

impl VexDetails {
    pub const fn new() -> Self {
        Self {
            pp: 0,
            map_select: 0,
            vex_we: false,
            vlength: MegaBool::set(None),
        }
    }
    pub const fn vlength(mut self, b: Option<bool>) -> Self {
        self.vlength = MegaBool::set(b);
        self
    }
    pub const fn vex_we(mut self, b: bool) -> Self {
        self.vex_we = b;
        self
    }
    pub const fn map_select(mut self, u: u8) -> Self {
        self.map_select = u;
        self
    }
    pub const fn pp(mut self, u: u8) -> Self {
        self.pp = u;
        self
    }
}

impl MegaBool {
    pub const fn from_byte(b: u8) -> Self {
        Self { data: b }
    }
    pub const fn set(op: Option<bool>) -> Self {
        Self {
            data: (op.is_some() as u8) << 1 | if let Some(b) = op { b } else { false } as u8,
        }
    }
    pub const fn get(&self) -> Option<bool> {
        if self.data & 0b0000_0010 == 0b10 {
            Some(self.data & 0b01 == 0b01)
        } else {
            None
        }
    }
}

impl OperandOrder {
    pub const fn new(op: &[OpOrd]) -> Option<Self> {
        if op.len() > 4 {
            None
        } else {
            let mut val = 0;
            let mut idx = 0;
            while idx < op.len() {
                val |= (op[idx] as u8) << (idx << 1);
                idx += 1;
            }
            Some(Self { ord: val })
        }
    }
    pub const fn get(&self, idx: u8) -> Option<OpOrd> {
        if idx > 4 {
            None
        } else {
            unsafe {
                // multiply by 2
                if idx == 0 {
                    return Some(std::mem::transmute::<u8, OpOrd>(self.ord & 0b11));
                }
                let opr = (self.ord >> (idx << 1)) & 0b11;
                Some(std::mem::transmute::<u8, OpOrd>(opr))
            }
        }
    }
    pub const fn modrm_reg_is_dst(&self) -> bool {
        self.ord & 0b00_00_00_11 == OpOrd::MODRM_REG as u8
    }
    pub fn deserialize(&self) -> [OpOrd; 4] {
        let mut arr = [OpOrd::MODRM_RM; 4];
        for (idx, it) in arr.iter_mut().enumerate() {
            *it = self.get(idx as u8).unwrap();
        }
        arr
    }
}

impl ModrmTuple {
    pub fn new(reg: Option<u8>) -> Self {
        Self {
            data: (reg.is_some() as u8) << 7 | reg.unwrap_or(0) << 3,
        }
    }
    pub const fn reg(&self) -> Option<u8> {
        if self.data & 0b10_000000 == 0b10_000000 {
            Some((self.data & 0b00_111000) >> 3)
        } else {
            None
        }
    }
    pub const fn deserialize(&self) -> Option<u8> {
        self.reg()
    }
    pub const fn data(&self) -> u8 {
        self.data
    }
}

impl Default for GenAPI {
    fn default() -> Self {
        Self::new()
    }
}

const fn pp(v: u8) -> u8 {
    match v {
        0x66 => 0b01,
        0xF3 => 0b10,
        0xF2 => 0b11,
        _ => 0,
    }
}
const fn map_select(v: u8) -> u8 {
    match v {
        0x0F => 0b00001,
        0x38 => 0b00010,
        0x3A => 0b00011,
        _ => 0b00000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tgeneral_api_check_0() {
        assert!(size_of::<GenAPI>() == 16);
    }
    #[test]
    fn tmbool_1() {
        use OpOrd::*;
        let mb = MegaBool::from_byte(3);
        assert_eq!(mb.get(), Some(true));
        let mb = MegaBool::set(Some(true));
        assert_eq!(mb.get(), Some(true));
        let api = GenAPI::new().vex(
            VexDetails::new()
                .map_select(0x0F)
                .pp(0x66)
                .vex_we(true)
                .vlength(Some(true)),
        );
        assert_eq!(api.get_fpfx(), PREFIX_VEX);
        assert_eq!(api.get_vex_vlength(), Some(MegaBool::set(Some(true))));
        assert!(api.get_vex_vlength().unwrap().get().unwrap_or(false));
        let api = GenAPI::new()
            .opcode(&[0x19])
            .modrm(true, None)
            .imm_atindex(2, 1)
            .vex(
                VexDetails::new()
                    .map_select(0x3A)
                    .pp(0x66)
                    .vex_we(false)
                    .vlength(Some(true)),
            )
            .ord(&[MODRM_RM, MODRM_REG]);
        assert_eq!(api.addt, 0b0000_0011_0001_0010);
        assert!(api.get_vex_vlength().unwrap().get().unwrap_or(false));
    }
    #[test]
    fn tord_check_2() {
        use OpOrd::*;
        assert!(MODRM_RM as u8 == 0);
        assert!(MODRM_REG as u8 == 1);
        assert!(VEX_VVVV as u8 == 2);
        assert!(TSRC as u8 == 3);
        let ord = OperandOrder::new(&[MODRM_RM, MODRM_REG, VEX_VVVV]).unwrap();
        assert!(ord.get(0) == Some(MODRM_RM));
        assert!(ord.get(1) == Some(MODRM_REG));
        assert!(ord.get(2) == Some(VEX_VVVV));
        assert!(ord.deserialize()[0..3] == [MODRM_RM, MODRM_REG, VEX_VVVV]);
    }
}
