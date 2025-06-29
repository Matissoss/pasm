// pasm - src/core/api.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

pub const REX_PFX: u8 = 0x0;
pub const VEX_PFX: u8 = 0x1;
pub const EVEX_PFX: u8 = 0x2;
pub const CAN_H66O: u8 = 0x3; // can use 0x66 override
pub const IMM_LEBE: u8 = 0x4; // immediate must be formatted as little endian or big endian; 0 = le, 1 = be
pub const CAN_SEGM: u8 = 0x5; // can use segment override
pub const USE_MODRM: u8 = 0x6; // can use modrm
pub const OBY_CONST: u8 = 0x7; // one byte const - 1st addt B goes as metadata, 2nd as immediate
pub const TBY_CONST: u8 = 0x8; // two byte const
pub const IMM_ATIDX: u8 = 0x9; // immediate at index (second byte of addt is index, first one is size)
pub const SET_MODRM: u8 = 0xA; // MODRM.mod is set to byte specified in addt2
pub const STRICT_PFX: u8 = 0xB; // makes all prefixes exclusive (e.g. if REX isn't set, then it cannot be generated)
pub const FIXED_SIZE: u8 = 0xC;

#[allow(unused)]
pub const EXT_FLGS1: u8 = 0xD;
#[allow(unused)]
pub const EXT_FLGS2: u8 = 0xE; // addt is BoolTable16

//
// Extended flags combo (not used):
// - EXT_FLGS1 : addt2 is BoolTable8 (+8 flags)
// - EXT_FLGS2 : addt is BoolTable16 (+16 flags)
// - EXT_FLGS1 + EXT_FLGS2: reserved
// - EXT_FLGSx + !CAN_SEGM: reserved
// - EXT_FLGSx + !CAN_H67O: reserved
// - [...]: reserved

use crate::{
    core::{disp, evex, modrm, rex, sib, vex},
    shr::{
        ast::{Instruction, Operand},
        booltable::BoolTable16,
        ins::Mnemonic,
        mem::Mem,
        reg::Register,
        reloc::Relocation,
        size::Size,
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

#[repr(transparent)]
struct Opcode {
    opcode: u64,
}

// size = 16B (could be 37B!)
#[repr(C)]
pub struct GenAPI {
    opcode: Opcode,

    flags: BoolTable16,

    // if (E)VEX_PFX flag is set, the byte is 0bX_YYYYY_ZZ where:
    // X = (E)VEX.w/e,
    // YYYYY = map_select
    // ZZ = pp
    // otherwise normal prefix like 0xF2/0xF3
    prefix: u8,

    // less essential - can be used with other context depending on flags

    // can be used with other context if USE_MODRM flag is NOT set
    modrm_ovr: ModrmTuple,

    // can be used with other context if USE_MODRM flag is NOT set (because why would you need it?)
    ord: OperandOrder,

    // - FIXED_SIZE - reserved for size
    // - SET_MODRM  - reserved for modrm.mod
    addt2: u8,

    // depending on flags:
    // - IMM_ATIDX, OBY_CONST, TBY_CONST - immediate + metadata,
    // - (E)VEX_PFX - first byte (last 2 bits) is reserved for vlength (and is cleared during .assemble()
    // otherwise unused
    addt: u16,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ModrmTuple {
    data: u8, // 0bXX_YYY_ZZZ : X1 = reg is Some, X2 = rm is Some, YYY = reg, ZZZ = rm
}

impl GenAPI {
    pub fn new() -> Self {
        Self {
            opcode: Opcode { opcode: 0 },
            prefix: 0,
            flags: BoolTable16::new().setc(CAN_H66O, true).setc(CAN_SEGM, true),
            ord: OperandOrder::new(&[
                OpOrd::MODRM_RM,
                OpOrd::MODRM_REG,
                OpOrd::VEX_VVVV,
                OpOrd::TSRC,
            ])
            .unwrap(),
            modrm_ovr: ModrmTuple::new(None, None),
            addt: 0,
            addt2: 0,
        }
    }
    pub const fn prefix(mut self, pfx: u8) -> Self {
        self.prefix = pfx;
        self
    }
    pub fn opcode(mut self, opc: &[u8]) -> Self {
        if opc.len() > 0b0000_0111 {
            panic!("Tried to use opcode of len() >= 8");
        }
        self.opcode = Opcode::from_bytes(opc);
        self
    }
    pub fn modrm(mut self, modrm: bool, reg: Option<u8>, rm: Option<u8>) -> Self {
        self.flags.set(USE_MODRM, modrm);
        self.modrm_ovr = ModrmTuple::new(reg, rm);
        self
    }
    pub const fn fixed_size(mut self, sz: Size) -> Self {
        self.flags.set(FIXED_SIZE, true);
        self.addt2 |= sz as u8;
        self
    }
    pub const fn modrm_mod(mut self, mod_: u8) -> Self {
        self.flags.set(SET_MODRM, true);
        self.addt2 = mod_ & 0b11;
        self
    }
    pub const fn can_h66(mut self, h66: bool) -> Self {
        self.flags.set(CAN_H66O, h66);
        self
    }
    pub const fn rex(mut self, rex: bool) -> Self {
        self.flags.set(REX_PFX, rex);
        self
    }
    pub const fn evex(mut self, vex_details: VexDetails) -> Self {
        self.flags.set(EVEX_PFX, true);
        self.prefix =
            (vex_details.vex_we as u8) << 7 | vex_details.map_select << 2 | pp(vex_details.pp);
        self.addt = ((vex_details.vlength.data as u16) << 0x08) | self.addt & 0x00FF;
        self
    }
    pub const fn vex(mut self, vex_details: VexDetails) -> Self {
        self.flags.set(VEX_PFX, true);
        self.prefix = {
            (vex_details.vex_we as u8) << 7
                | map_select(vex_details.map_select) << 2
                | pp(vex_details.pp)
        };
        self.addt = ((vex_details.vlength.data as u16) << 0x08) | self.addt & 0x00FF;
        self
    }
    pub const fn imm_atindex(mut self, idx: u16, size: u16) -> Self {
        self.flags.set(IMM_ATIDX, true);
        self.addt = ((size << 4) | idx & 0b1111) | self.addt & 0xFF00;
        self
    }
    pub const fn imm_is_be(mut self, bool: bool) -> Self {
        self.flags.set(IMM_LEBE, bool);
        self
    }
    pub const fn imm_const8(mut self, extend_to: u8, imm: u8) -> Self {
        self.flags.set(OBY_CONST, true);
        self.addt = (extend_to as u16) << 8 | imm as u16;
        self
    }
    pub const fn imm_const16(mut self, imm: u16) -> Self {
        self.flags.set(TBY_CONST, true);
        self.addt = imm;
        self
    }
    pub const fn ord(mut self, ord: &[OpOrd]) -> Self {
        self.ord = OperandOrder::new(ord).expect("Failed to create operand order");
        self
    }
    pub const fn get_flag(&self, idx: u8) -> Option<bool> {
        self.flags.get(idx)
    }
    pub const fn set_flag(mut self, idx: u8) -> Self {
        self.flags.set(idx, true);
        self
    }
    pub const fn get_addt2(&self) -> u8 {
        self.addt2
    }
    pub fn get_size(&self) -> Option<Size> {
        if self.flags.get(FIXED_SIZE).unwrap_or(false) {
            Some(unsafe { std::mem::transmute::<u8, Size>(self.addt2) })
        } else {
            None
        }
    }
    pub fn debug_assemble<'a>(
        &'a self,
        ins: &'a Instruction,
        bits: u8,
    ) -> (Vec<u8>, [Option<Relocation>; 2]) {
        let res = self.assemble(ins, bits);
        print!("LINE {:8}:", ins.line + 1);
        for b in &res.0 {
            print!(" {:02x}", b);
        }
        println!();
        res
    }
    // you can have max 2 relocations returned, because of variants like:
    // mov .deref @symbol, @other_symbol
    // (yes it is in fact a valid variant: first operand is mem, second is immediate)
    pub fn assemble<'a>(
        &'a self,
        ins: &'a Instruction,
        bits: u8,
    ) -> (Vec<u8>, [Option<Relocation>; 2]) {
        let mut rels = [None, None];
        let vex_flag_set = self.flags.get(VEX_PFX).unwrap();
        let rex_flag_set = self.flags.get(REX_PFX).unwrap();
        let evex_flag_set = self.flags.get(EVEX_PFX).unwrap();
        let (ins_size, fx_size) = if let Some(sz) = self.get_size() {
            (sz, true)
        } else {
            (ins.size(), false)
        };

        let mut base = {
            let mut base = Vec::new();

            if let Some(a) = gen_addt_pfx(ins) {
                base.push(a);
            }

            let rex = if (rex_flag_set || !(vex_flag_set || evex_flag_set))
                && !self.get_flag(STRICT_PFX).unwrap()
            {
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

            if !(vex_flag_set || evex_flag_set) {
                if let Some(segm) = gen_segm_pref(ins) {
                    base.push(segm);
                }
                if fx_size {
                    if let Some(size_ovr) = gen_sizeovr_fixed_size(ins_size, bits) {
                        base.push(size_ovr);
                    }
                } else if let Some(size_ovr) = gen_size_ovr(ins, bits, rexw) {
                    let h66 = size_ovr[0];
                    let h67 = size_ovr[1];
                    if h66.is_some() && self.prefix != 0x66 && self.get_flag(CAN_H66O).unwrap() {
                        base.push(0x66);
                    }
                    if h67.is_some() && self.prefix != 0x67 {
                        base.push(0x67);
                    }
                }
                if matches!(self.prefix, 0xF0 | 0xF2 | 0xF3 | 0x66) {
                    base.push(self.prefix);
                }
            }

            if rex_flag_set && rex != 0x00 {
                base.push(rex);
            }
            if vex_flag_set {
                if let Some(vex) = vex::vex(ins, self) {
                    base.extend(vex);
                }
            }
            if evex_flag_set {
                base.extend(evex::evex(self, ins));
            }
            base
        };

        let (opc, sz) = self.opcode.collect();
        base.extend(&opc[..sz]);
        if self.flags.get(USE_MODRM).unwrap() {
            base.push(modrm::modrm(ins, self));
            if let Some(sib) = sib::gen_sib_ins(ins) {
                base.push(sib);
            }
            if let Some(disp) = disp::gen_disp_ins(ins) {
                base.extend(disp);
            } else {
                let symb = ins.get_symbs()[0];
                if let Some((s, _)) = symb {
                    let addend = s.addend().unwrap_or_default();
                    rels[0] = Some(Relocation {
                        symbol: s.symbol.clone(),
                        offset: base.len() as u32,
                        addend,
                        shidx: 0,
                        reltype: s.reltype().unwrap_or_default(),
                    });
                    base.extend(vec![0; 4]);
                }
            }
        }
        if self.flags.get(IMM_ATIDX).unwrap() {
            let size = ((self.addt & 0x00_F0) >> 4) as usize;
            let idx = (self.addt & 0x00_0F) as usize;
            if let Some(Operand::Imm(i)) = ins.get_opr(idx) {
                let (imm, be) = if self.get_flag(IMM_LEBE).unwrap_or(false) {
                    (&i.get_raw_be()[8 - i.get_real_size()..], true)
                } else {
                    (&i.get_raw_le()[..i.get_real_size()], false)
                };
                let mut idx = 0;
                if be {
                    while idx < size.abs_diff(imm.len()) {
                        base.push(0x00);
                        idx += 1;
                    }
                }
                for b in imm {
                    if idx < size {
                        base.push(*b);
                        idx += 1;
                    } else {
                        break;
                    }
                }
                if !be {
                    while idx < size {
                        base.push(0x00);
                        idx += 1;
                    }
                }
            } else if let Some(Operand::String(s)) = ins.get_opr(idx) {
                if size == 0 {
                    base.extend(s.as_bytes());
                } else {
                    let mut bts = s.as_bytes().to_vec();
                    while bts.len() < size {
                        bts.push(0x00);
                    }
                    base.extend(bts);
                }
            } else if let Some(Operand::SymbolRef(s)) = ins.get_opr(idx) {
                let addend = s.addend().unwrap_or_default();
                rels[1] = Some(Relocation {
                    symbol: s.symbol.clone(),
                    offset: base.len() as u32,
                    addend,
                    shidx: 0,
                    reltype: s.reltype().unwrap_or_default(),
                });
                let sz: u8 = s.size().unwrap_or(Size::Dword).into();
                base.extend(vec![0; sz as usize]);
            }
            // rvrm
            else if let Some(Operand::Reg(r)) = ins.get_opr(idx) {
                let mut v = Vec::new();
                v.push((r.get_ext_bits()[1] as u8) << 7 | r.to_byte() << 4);
                extend_imm(&mut v, size as u8);
                base.extend(v);
            }
        } else if self.flags.get(OBY_CONST).unwrap() {
            let size = (self.addt & 0xFF00) >> 8;
            let mut imm = vec![((self.addt & 0x00FF) as u8)];
            extend_imm(&mut imm, size as u8);
            base.extend(imm);
        }

        for r in rels.iter_mut().flatten() {
            if r.is_rel() {
                r.addend -= base.len() as i32 - 1;
            }
        }

        (base, rels)
    }
    pub const fn modrm_reg_is_dst(&self) -> bool {
        self.ord.modrm_reg_is_dst()
    }
    pub const fn get_modrm(&self) -> ModrmTuple {
        self.modrm_ovr
    }
    pub fn get_ord(&self) -> [OpOrd; 4] {
        self.ord.deserialize()
    }
    #[rustfmt::skip]
    pub fn get_ord_oprs<'a>(&self, ins: &'a Instruction) -> [Option<&'a Operand>; 3] {
        use OpOrd::*;
        let ord = self.ord.deserialize();
        match &ord[..3] {
            //                                    MODRM.r/m   MODRM.reg   (E)VEX.vvvv
            [MODRM_REG, VEX_VVVV , MODRM_RM ] => [ins.src2(), ins.dst() , ins.src() ],
            [MODRM_RM , VEX_VVVV , MODRM_REG] => [ins.dst() , ins.src2(), ins.src() ],
            [VEX_VVVV , MODRM_REG, _        ] => [None      , ins.src() , ins.dst() ],
            [VEX_VVVV , MODRM_RM , _        ] => [ins.src() , None      , ins.dst() ],
            [MODRM_REG, MODRM_RM , _        ] => [ins.src() , ins.dst() , ins.src2()],
            [MODRM_RM , MODRM_REG, _        ] => [ins.dst() , ins.src() , ins.src2()],
            _                                 => [None      , None      , None      ],
        }
    }
    pub const fn strict_pfx(mut self) -> Self {
        self.flags.set(STRICT_PFX, true);
        self
    }
    // fails if (E)VEX flag is not set
    pub const fn get_pp(&self) -> Option<u8> {
        if self.flags.get(VEX_PFX).unwrap() || self.flags.get(EVEX_PFX).unwrap() {
            Some(self.prefix & 0b11)
        } else {
            None
        }
    }
    // fails if (E)VEX flag is not set
    pub const fn get_map_select(&self) -> Option<u8> {
        if self.flags.get(VEX_PFX).unwrap() || self.flags.get(EVEX_PFX).unwrap() {
            Some((self.prefix & 0b0111_1100) >> 2)
        } else {
            None
        }
    }
    // fails if (E)VEX flag is not set
    pub const fn get_vex_we(&self) -> Option<bool> {
        if self.flags.get(VEX_PFX).unwrap() || self.flags.get(EVEX_PFX).unwrap() {
            Some(self.prefix & 0b1000_0000 == 0b1000_0000)
        } else {
            None
        }
    }
    // fails if (E)VEX flag is not set
    pub const fn get_vex_vlength(&self) -> Option<MegaBool> {
        if self.flags.get(VEX_PFX).unwrap() || self.flags.get(EVEX_PFX).unwrap() {
            Some(MegaBool::from_byte(((self.addt & 0xFF00) >> 8) as u8))
        } else {
            None
        }
    }
}

fn gen_addt_pfx(ins: &Instruction) -> Option<u8> {
    use Mnemonic as Ins;
    if let Some(s) = ins.addt {
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

fn gen_size_ovr(ins: &Instruction, bits: u8, rexw: bool) -> Option<[Option<u8>; 2]> {
    let mut arr = [None; 2];
    if ins.dst().is_some() && ins.src().is_none() {
        let dst = ins.dst().unwrap();
        if let Operand::Imm(_) = dst {
            return None;
        }
    }
    match bits {
        16 => {
            if let Size::Dword = ins.size() {
                arr[0] = Some(0x66);
            }
        }
        32 => {
            if let Size::Word = ins.size() {
                arr[0] = Some(0x66);
            }
        }
        64 => match ins.size() {
            Size::Word => arr[0] = Some(0x66),
            Size::Qword => {
                if !(rexw || ins.mnem.defaults_to_64bit() || ins.uses_cr() || ins.uses_dr()) {
                    arr[0] = Some(0x66);
                }
            }
            _ => {}
        },
        _ => {}
    };
    if let Some(m) = ins.get_mem() {
        match (m.addrsize().unwrap_or(Size::Unknown), bits) {
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
        return gen_segm_pref_op(mem);
    }
    None
}

fn extend_imm(imm: &mut Vec<u8>, size: u8) {
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
    pub fn new(reg: Option<u8>, rm: Option<u8>) -> Self {
        Self {
            data: ((reg.is_some() as u8) << 7
                | (rm.is_some() as u8) << 6
                | reg.unwrap_or(0) << 3
                | rm.unwrap_or(0)),
        }
    }
    pub const fn rm(&self) -> Option<u8> {
        if self.data & 0b01_000000 == 0b01_000000 {
            Some(self.data & 0b000111)
        } else {
            None
        }
    }
    pub const fn reg(&self) -> Option<u8> {
        if self.data & 0b10_000000 == 0b10_000000 {
            Some((self.data & 0b00_111000) >> 3)
        } else {
            None
        }
    }
    pub const fn deserialize(&self) -> (Option<u8>, Option<u8>) {
        (self.reg(), self.rm())
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

impl Opcode {
    fn from_bytes(bytes: &[u8]) -> Self {
        let size = bytes.len();
        assert!(size <= 0b110);
        let opcode: u64 = {
            (*bytes.first().unwrap_or(&0) as u64) << 56
                | (*bytes.get(1).unwrap_or(&0) as u64) << 48
                | (*bytes.get(2).unwrap_or(&0) as u64) << 40
                | (*bytes.get(3).unwrap_or(&0) as u64) << 32
                | (*bytes.get(4).unwrap_or(&0) as u64) << 24
                | (*bytes.get(5).unwrap_or(&0) as u64) << 16
                | (*bytes.get(6).unwrap_or(&0) as u64) << 8
                | (size as u64 & 0xFF)
        };
        Self { opcode }
    }
    const fn collect(&self) -> ([u8; 8], usize) {
        let size = (self.opcode & 0x0000_0000_0000_00FF) as usize;
        let opc = self.opcode & 0xFFFF_FFFF_FFFF_FF00;
        (opc.to_be_bytes(), size)
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
    fn general_api_check() {
        assert!(size_of::<GenAPI>() == 16);
    }
    #[test]
    fn mbool() {
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
        assert_eq!(api.get_vex_vlength(), Some(MegaBool::set(Some(true))));
        assert!(api.get_vex_vlength().unwrap().get().unwrap_or(false));
        let api = GenAPI::new()
            .opcode(&[0x19])
            .modrm(true, None, None)
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
    fn ord_check() {
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
