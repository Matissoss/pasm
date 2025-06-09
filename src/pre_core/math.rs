// rasmx86_64 - src/pre_core/math.rs
// ---------------------------------
// made by matissoss
// licensed under MPL 2.0
use std::collections::HashMap;

use crate::shr::{
    ast::{Label, Operand, AST},
    error::RASMError,
    math::MathematicalEvaluation as MathEval,
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
            math_symbols.insert(&m.0, &m.1);
        }
    }
    for l in &mut ast.labels {
        replace_mathevals(l, &math_symbols)?;
    }
    Ok(())
}

pub fn replace_mathevals(
    label: &mut Label,
    mth: &HashMap<&String, &String>,
) -> Result<(), RASMError> {
    for i in &mut label.inst {
        for o in &mut i.oprs {
            if let Some(Operand::SymbolRef(s)) = o {
                if mth.contains_key(&*s) {
                    let eval = {
                        let e = mth.get(&*s).unwrap();
                        if let Ok(n) = Number::from_str(e) {
                            n.get_as_u64()
                        } else {
                            let eval = MathEval::from_str(e)?;
                            let n = MathEval::eval(eval);
                            if n.is_none() {
                                return Err(RASMError::with_tip(None,
                                    Some(format!("Couldn't evaluate symbol {s}, despite it being declared as mathematical symbol")),
                                    Some("If this helps: you cannot reference other mathematical symbols"),
                                ));
                            }
                            n.unwrap()
                        }
                    };
                    *o = Some(Operand::Imm(Number::uint64(eval)));
                }
            }
        }
    }
    Ok(())
}
