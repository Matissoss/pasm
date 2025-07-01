// pasm - src/pre_core/math.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0
use std::collections::HashMap;

use crate::shr::{
    ast::{Label, Operand, AST},
    error::RError as Error,
    num::Number,
};

pub fn post_process(ast: &mut AST) -> Result<(), Error> {
    for sec in &mut ast.sections {
        for l in &mut sec.content {
            replace_mathevals(l, &ast.defined)?;
        }
    }
    Ok(())
}

pub fn replace_mathevals(
    label: &mut Label,
    mth: &HashMap<crate::RString, u64>,
) -> Result<(), Error> {
    for i in &mut label.inst {
        for o in i.operands.iter_mut() {
            if let Operand::SymbolRef(s) = o {
                if mth.contains_key(&s.symbol) {
                    let eval = mth.get(&s.symbol).unwrap();
                    *o = Operand::Imm(Number::uint64(*eval));
                }
            }
        }
    }
    Ok(())
}
