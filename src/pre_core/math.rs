// pasm - src/pre_core/math.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0
use std::collections::HashMap;

use crate::shr::{
    ast::{Operand, AST},
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

pub fn replace_mathevals(
    label: &mut Label,
    mth: &HashMap<crate::RString, Number>,
) -> Result<(), Error> {
    for i in &mut label.content {
        for o in i.operands.iter_mut() {
            if let Operand::SymbolRef(s) = o {
                if mth.contains_key(&s.symbol) {
                    let eval = mth.get(&s.symbol).unwrap();
                    *o = Operand::Imm(Number::uint64(eval.get_as_u64()));
                }
            }
        }
    }
    Ok(())
}
