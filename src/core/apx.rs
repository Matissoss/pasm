// pasm - src/core/apx.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    core::api::GenAPI,
    shr::{ast::Instruction, smallvec::SmallVec},
};

// TODO: implement APX support
pub fn apx(_ctx: &GenAPI, _ins: &Instruction) -> SmallVec<u8, 4> {
    todo!()
}

#[inline(always)]
fn _rex2() {
    todo!()
}
#[inline(always)]
fn _eevex_legacy() {
    todo!()
}
#[inline(always)]
fn _eevex_cond() {
    todo!()
}
#[inline(always)]
fn _eevex_evex() {
    todo!()
}
#[inline(always)]
fn _eevex_vex() {
    todo!()
}
