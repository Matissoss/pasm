// pasm - src/core/apx.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

// ATTENTION:
//  APX in pasm may not work correctly, but i blame other assemblers for that :D
//  I have no way validating if it's correct, so we have to pray that this awful code works
//
//  - matissoss

use crate::{
    core::api::GenAPI,
    shr::{
        ast::{Instruction, Operand},
        size::Size,
        smallvec::SmallVec,
    },
};

#[repr(u8)]
#[derive(PartialEq)]
pub enum APXVariant {
    Auto = 0b000,
    VexExtension = 0b001,
    LegacyExtension = 0b010,
    CondTestCmpExtension = 0b011,
    EvexExtension = 0b100,
    Rex2 = 0b101,
}

pub fn apx(ctx: &GenAPI, ins: &Instruction, bits: u8) -> SmallVec<u8, 4> {
    match ctx.get_apx_eevex_version() {
        Some(APXVariant::EvexExtension) => eevex_evex(ctx, ins),
        Some(APXVariant::VexExtension) => {
            if ins.needs_evex() {
                eevex_evex(ctx, ins)
            } else {
                eevex_vex(ctx, ins)
            }
        }
        Some(APXVariant::Auto) => todo!("Either rex2 or eevex"),
        Some(APXVariant::CondTestCmpExtension) => eevex_cond(ctx, ins),
        Some(APXVariant::Rex2) => rex2(ctx, ins),
        Some(APXVariant::LegacyExtension) => eevex_legacy(ctx, ins, bits),
        _ => panic!(
            "why are we invoking crate::core::apx::apx, when GenAPI does not have .apx()? {ins:?}"
        ),
    }
}

#[inline(always)]
fn rex2(_ctx: &GenAPI, _ins: &Instruction) -> SmallVec<u8, 4> {
    todo!()
}
#[inline(always)]
fn eevex_legacy(ctx: &GenAPI, ins: &Instruction, bits: u8) -> SmallVec<u8, 4> {
    let mut vec = SmallVec::<u8, 4>::new();

    let [modrm_rm, modrm_reg, evex_vvvv] = ctx.get_ord_oprs(ins);

    let [[evex_r4, evex_r3], [_, _]] = ebits(&modrm_reg);
    let [[evex_b4, evex_b3], [evex_x4, evex_x3]] = ebits(&modrm_rm);
    let [[evex_v4, _], [_, _]] = ebits(&evex_vvvv);

    let (evex_we, evex_pp) = match (ins.size(), bits) {
        (Size::Qword, 64) => (ctx.get_apx_eevex_vex_we(), 0),
        (Size::Qword, _) => (ctx.get_apx_eevex_vex_we(), 0b01),
        (Size::Dword, 32) => (ctx.get_apx_eevex_vex_we(), 0),
        (Size::Dword, _) => (ctx.get_apx_eevex_vex_we(), 0b01),
        _ => (ctx.get_apx_eevex_vex_we(), ctx.get_apx_eevex_pp()),
    };

    vec.push(0x62);
    vec.push(
        (!evex_r3 as u8) << 7
            | (!evex_x3 as u8) << 6
            | (!evex_b3 as u8) << 5
            | (!evex_r4 as u8) << 4
            | (!evex_b4 as u8) << 3
            | 1 << 2,
    );

    vec.push((evex_we as u8) << 7 | gen_evex4v(&evex_vvvv) << 3 | (!evex_x4 as u8) << 2 | evex_pp);
    vec.push(
        (ins.apx_get_leg_nd().unwrap_or(false) as u8) << 4
            | (!evex_v4 as u8) << 3
            | (ins.apx_get_leg_nf().unwrap_or(false) as u8) << 2,
    );
    vec
}
#[inline(always)]
fn eevex_cond(_ctx: &GenAPI, _ins: &Instruction) -> SmallVec<u8, 4> {
    todo!()
}

// TODO: apparently there might be something wrong with this,
//       because for {apx-evex} vaddps xmm21, xmm23, xmm24
//       we get for first byte 0xC1, when 0x81 is the correct

// NOTE: intel's APX documentation says something about EVEX.U field, but
//       idk what it should be set to, when ModRM.mod == 0b11
#[inline(always)]
fn eevex_evex(ctx: &GenAPI, ins: &Instruction) -> SmallVec<u8, 4> {
    let mut vec = SmallVec::<u8, 4>::new();

    let [modrm_rm, modrm_reg, evex_vvvv] = ctx.get_ord_oprs(ins);

    let [[evex_r4, evex_r3], [_, _]] = ebits(&modrm_reg);
    let [[evex_b4, evex_b3], [evex_x4, evex_x3]] = ebits(&modrm_rm);
    let [[evex_v4, _], [_, _]] = ebits(&evex_vvvv);

    vec.push(0x62);
    vec.push(
        (!evex_r3 as u8) << 7
            | (!evex_x3 as u8) << 6
            | (!evex_b3 as u8) << 5
            | (!evex_r4 as u8) << 4
            | (!evex_b4 as u8) << 3
            | ctx.get_map_select().unwrap_or(0) & 0b111,
    );

    vec.push(
        (ctx.get_vex_we().unwrap_or(false) as u8) << 7
            | gen_evex4v(&evex_vvvv) << 3
            | (!evex_x4 as u8) << 2
            | ctx.get_pp().unwrap_or(0b00),
    );
    vec.push(
        ((ins.size() == Size::Yword) as u8) << 4
            | (!evex_v4 as u8) << 3
            | (ins.apx_eevex_vex_get_nf().unwrap_or(false) as u8) << 2,
    );
    vec
}
#[inline(always)]
fn eevex_vex(ctx: &GenAPI, ins: &Instruction) -> SmallVec<u8, 4> {
    let mut vec = SmallVec::<u8, 4>::new();

    let [modrm_rm, modrm_reg, evex_vvvv] = ctx.get_ord_oprs(ins);

    let [[evex_r3, evex_r4], [_, _]] = ebits(&modrm_reg);
    let [[evex_b3, evex_b4], [evex_x3, evex_x4]] = ebits(&modrm_rm);
    let [[evex_v4, _], [_, _]] = ebits(&evex_vvvv);

    let (bcst, sz) = if let Some(Operand::Mem(m)) = modrm_rm {
        (m.is_bcst(), m.size())
    } else {
        (false, Size::Unknown)
    };
    let mut evex3 = {
        (ins.evex_z().unwrap_or(false) as u8) << 7
            | ((ins.evex_sae().unwrap_or(false) || bcst) as u8) << 4
            | (!evex_v4 as u8) << 3
            | ins.evex_mask().unwrap_or(0)
    };
    if let Some(er) = ins.evex_er() {
        evex3 |= 1 << 4;
        evex3 |= er << 5;
    } else {
        let isz = ins.size();
        evex3 |= ((isz == Size::Zword) as u8) << 6 | ((isz == Size::Yword) as u8) << 5
    }

    let evex_we = if ctx.get_vex_we() == Some(true) {
        true
    } else if bcst {
        sz == Size::Qword
    } else {
        false
    };

    vec.push(0x62);
    vec.push(
        (!evex_r3 as u8) << 7
            | (!evex_x3 as u8) << 6
            | (!evex_b3 as u8) << 5
            | (!evex_r4 as u8) << 4
            | (!evex_b4 as u8) << 3
            | ctx.get_map_select().unwrap_or(0) & 0b111,
    );

    vec.push(
        (evex_we as u8) << 7 |
        gen_evex4v(&evex_vvvv) << 3 |
        // this is the EEVEX.U field
        (!evex_x4 as u8) << 2 |
        ctx.get_pp().unwrap_or(0b00),
    );

    vec.push(evex3);
    vec
}

fn gen_evex4v(op: &Option<Operand>) -> u8 {
    if let Some(Operand::Register(r)) = op {
        crate::utils::andn((r.ebits()[1] as u8) << 3 | r.to_byte(), 0b0000_1111)
    } else {
        0b1111
    }
}

fn ebits(op: &Option<Operand>) -> [[bool; 2]; 2] {
    if let Some(op) = op {
        op.ebits()
    } else {
        [[false; 2]; 2]
    }
}
