// pasm - src/pre_core/math.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0
use std::collections::HashMap;

use crate::shr::{
    ast::{Operand, OperandOwned, AST},
    error::Error,
    label::Label,
    num::Number,
};

pub fn post_process(ast: &mut AST) -> Result<(), Error> {
    for sec in &mut ast.sections {
        for l in &mut sec.content {
            replace_mathevals(l, &ast.defines)?;
        }
    }
    Ok(())
}

pub fn replace_mathevals(label: &mut Label, mth: &HashMap<&str, Number>) -> Result<(), Error> {
    for i in &mut label.content {
        if i.is_empty() {
            continue;
        }
        for idx in 0..i.len() {
            if let Some(Operand::Symbol(s)) = i.get(idx) {
                if let Some(v) = mth.get(&s.symbol) {
                    i.set(idx, OperandOwned::Imm(Number::uint64(v.get_as_u64())));
                }
            }
        }
    }
    Ok(())
}
