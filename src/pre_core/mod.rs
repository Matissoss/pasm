// rasmx86_64 - src/pre_core/mod.rs
// --------------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{ast::AST, error::RASMError};

pub mod math;

// here we run code before assembling phase is started
pub fn post_process(ast: &mut AST) -> Result<(), RASMError> {
    math::post_process(ast)?;
    Ok(())
}
