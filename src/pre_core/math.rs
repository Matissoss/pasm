// rasmx86_64 - src/pre_core/math.rs
// ---------------------------------
// made by matissoss
// licensed under MPL 2.0
use std::collections::HashMap;

use crate::shr::{
    ast::{Label, Operand, AST},
    error::RASMError,
    num::Number,
};

pub fn post_process(ast: &mut AST) -> Result<(), RASMError> {
    let mut math_symbols = HashMap::new();
    for m in &mut ast.math {
        if math_symbols.contains_key(&m.0) {
            return Err(RASMError::no_tip(
                None,
                Some(format!(
                    "Symbol with same name (`{}`) was already declared",
                    &m.0
                )),
            ));
        } else {
            math_symbols.insert(&m.0, m.1);
        }
    }
    for sec in &mut ast.sections {
        for l in &mut sec.content {
            replace_mathevals(l, &math_symbols)?;
        }
    }
    Ok(())
}

pub fn replace_mathevals(
    label: &mut Label,
    mth: &HashMap<&crate::RString, u64>,
) -> Result<(), RASMError> {
    for i in &mut label.inst {
        for o in &mut i.oprs {
            if let Some(Operand::SymbolRef(s)) = o {
                if mth.contains_key(&s.symbol) {
                    let eval = mth.get(&s.symbol).unwrap();
                    *o = Some(Operand::Imm(Number::uint64(*eval)));
                }
            }
        }
    }
    Ok(())
}
