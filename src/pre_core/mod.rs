// pasm - src/pre_core/mod.rs
// --------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{ast::AST, error::Error};

pub mod math;

// here we run code before assembling phase is started
pub fn post_process(ast: &mut AST) -> Result<(), Error> {
    math::post_process(ast)?;
    Ok(())
}
