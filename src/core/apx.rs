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
        instruction::{Instruction, Operand},
        size::Size,
        stackvec::StackVec,
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

pub fn apx(ctx: &GenAPI, ins: &Instruction, bits: u8) -> StackVec<u8, 4> {
    match ctx.get_apx_eevex_version() {
        Some(APXVariant::EvexExtension) => eevex_evex(ctx, ins),
        Some(APXVariant::VexExtension) => {
            if ins.needs_evex() {
                eevex_evex(ctx, ins)
            } else {
                eevex_vex(ctx, ins)
            }
        }
        Some(APXVariant::Auto) => {
            if ctx.get_apx_eevex_map_select() <= 1 && ctx.get_apx_eevex_pp() == 0 {
                if bits == 64 {
                    rex2(ctx, ins)
                } else if ins.needs_evex() {
                    eevex_evex(ctx, ins)
                } else if ins.which_variant() != crate::shr::instruction::IVariant::STD {
                    eevex_vex(ctx, ins)
                } else {
                    eevex_legacy(ctx, ins, bits)
                }
            } else if ins.needs_evex() {
                eevex_evex(ctx, ins)
            } else if ins.which_variant() != crate::shr::instruction::IVariant::STD {
                eevex_vex(ctx, ins)
            } else {
                eevex_legacy(ctx, ins, bits)
            }
        }
        Some(APXVariant::CondTestCmpExtension) => eevex_cond(ctx, ins),
        Some(APXVariant::Rex2) => rex2(ctx, ins),
        Some(APXVariant::LegacyExtension) => eevex_legacy(ctx, ins, bits),
        _ => panic!(
            "why are we invoking crate::core::apx::apx, when GenAPI does not have .apx()? {ins:?}"
        ),
    }
}

#[inline(always)]
fn rex2(ctx: &GenAPI, ins: &Instruction) -> StackVec<u8, 4> {
    let mut vec = StackVec::new();

    vec.push(0xD5);

    let [modrm_rm, modrm_reg, _] = ctx.get_ord_oprs(ins);

    let [[evex_r4, evex_r3], [_, _]] = ebits(&modrm_reg);
    let [[evex_b4, evex_b3], [evex_x4, evex_x3]] = ebits(&modrm_rm);

    let mut rex2_0 = (ctx.get_apx_eevex_map_select() & 1) << 7
        | (evex_r4 as u8) << 6
        | (ctx.get_apx_eevex_vex_we() as u8) << 3
        | (evex_r3 as u8) << 2;

    if let Some(Operand::Mem(_)) = modrm_rm {
        rex2_0 |= evex_b3 as u8;
        rex2_0 |= (evex_x3 as u8) << 1;
        rex2_0 |= (evex_b4 as u8) << 4;
        rex2_0 |= (evex_x4 as u8) << 5;
    } else {
        rex2_0 |= evex_b3 as u8;
        rex2_0 |= (evex_b4 as u8) << 1;
    }

    vec.push(rex2_0);

    vec
}
#[inline(always)]
fn eevex_legacy(ctx: &GenAPI, ins: &Instruction, bits: u8) -> StackVec<u8, 4> {
    let mut vec = StackVec::<u8, 4>::new();

    let [modrm_rm, modrm_reg, evex_vvvv] = ctx.get_ord_oprs(ins);

    let [[evex_r4, evex_r3], [_, _]] = ebits(&modrm_reg);
    let [[evex_b4, evex_b3], [evex_x4, evex_x3]] = ebits(&modrm_rm);
    let [[evex_v4, _], [_, _]] = ebits(&evex_vvvv);

    let (evex_we, evex_pp) = if ctx.get_apx_eevex_pp() != 0 {
        (ins.size() == Size::Qword, ctx.get_apx_eevex_pp())
    } else {
        match (ins.size(), bits) {
            (Size::Qword, 64) => (true, 0),
            (Size::Dword, 64) => (false, 0),
            (Size::Word, 64) => (false, 0b01),

            (Size::Dword, 32) => (false, 0),
            (Size::Word, 32) => (false, 0b01),

            (Size::Dword, 16) => (false, 0b01),
            (Size::Word, 16) => (false, 0),
            _ => (ctx.get_apx_eevex_vex_we(), ctx.get_apx_eevex_pp()),
        }
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
        (evex_vvvv.is_some() as u8) << 4
            | (!evex_v4 as u8) << 3
            | (ins.apx_get_leg_nf().unwrap_or(false) as u8) << 2,
    );
    vec
}
#[inline(always)]
fn eevex_cond(ctx: &GenAPI, ins: &Instruction) -> StackVec<u8, 4> {
    let mut vec = StackVec::<u8, 4>::new();

    let [modrm_rm, modrm_reg, _] = ctx.get_ord_oprs(ins);

    let [[evex_r4, evex_r3], [_, _]] = ebits(&modrm_reg);
    let [[evex_b4, evex_b3], [evex_x4, evex_x3]] = ebits(&modrm_rm);

    vec.push(0x62);
    vec.push(
        (!evex_r3 as u8) << 7
            | (!evex_x3 as u8) << 6
            | (!evex_b3 as u8) << 5
            | (!evex_r4 as u8) << 4
            | (!evex_b4 as u8) << 3
            | 1 << 2,
    );

    vec.push(
        (ctx.get_apx_eevex_vex_we() as u8) << 7
            | (ins.apx_eevex_cond_get_of().unwrap_or(false) as u8) << 6
            | (ins.apx_eevex_cond_get_sf().unwrap_or(false) as u8) << 6
            | (ins.apx_eevex_cond_get_zf().unwrap_or(false) as u8) << 6
            | (ins.apx_eevex_cond_get_cf().unwrap_or(false) as u8) << 6
            | (!evex_x4 as u8) << 2
            | ctx.get_apx_eevex_pp(),
    );
    vec.push(ctx.get_apx_cccc() & 0b1111);
    vec
}

#[inline(always)]
fn eevex_vex(ctx: &GenAPI, ins: &Instruction) -> StackVec<u8, 4> {
    let mut vec = StackVec::<u8, 4>::new();

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

// NOTE: intel's APX documentation says something about EVEX.U field, but
//       idk what it should be set to, when ModRM.mod == 0b11
#[inline(always)]
fn eevex_evex(ctx: &GenAPI, ins: &Instruction) -> StackVec<u8, 4> {
    let mut vec = StackVec::<u8, 4>::new();

    let [modrm_rm, modrm_reg, evex_vvvv] = ctx.get_ord_oprs(ins);

    let [[evex_r4, evex_r3], [_, _]] = ebits(&modrm_reg);
    let [[evex_b4, evex_b3], [evex_x4, evex_x3]] = ebits(&modrm_rm);
    let [[evex_v4, _], [_, _]] = ebits(&evex_vvvv);

    let (bcst, sz, mem) = if let Some(Operand::Mem(m)) = modrm_rm {
        (m.is_bcst(), m.size(), true)
    } else {
        (false, Size::Unknown, false)
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

    let mut evex1 =
        (!evex_r3 as u8) << 7 | (!evex_r4 as u8) << 4 | ctx.get_map_select().unwrap_or(0) & 0b111;

    let mut evex2 =
        (evex_we as u8) << 7 | gen_evex4v(&evex_vvvv) << 3 | ctx.get_pp().unwrap_or(0b00);

    // set these fields if we use mem (B = Base, X = Index)
    if mem {
        evex1 |= (!evex_x3 as u8) << 6;
        evex1 |= (!evex_b3 as u8) << 5;
        evex1 |= (!evex_b4 as u8) << 3;

        // this is the EEVEX.U field
        evex2 |= (!evex_x4 as u8) << 2;
    }
    // otherwise just extension of ModRM.r/m register
    else {
        evex1 |= (!evex_b4 as u8) << 6;
        evex1 |= (!evex_b3 as u8) << 5;
    }

    vec.push(0x62);
    vec.push(evex1);
    vec.push(evex2);
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
