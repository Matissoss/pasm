// rasmx86_64 - src/shr/rpanic.rs
// ------------------------------
// made by matissoss
// licensed under MPL 2.0

use std::panic;

use crate::color::{BaseColor, ColString};

pub fn switch_panichandler() {
    panic::set_hook(Box::new(rpanic_rs));
}

fn rpanic_rs(panic: &panic::PanicHookInfo) {
    let content = {
        if let Some(str) = panic.payload().downcast_ref::<String>() {
            str.to_string()
        } else if let Some(str) = panic.payload().downcast_ref::<&str>() {
            str.to_string()
        } else if let Some(numb) = panic.payload().downcast_ref::<i32>() {
            numb.to_string()
        } else {
            "".to_string()
        }
    };
    let location = panic.location();

    eprintln!(
        "{}\n\t{}!{}",
        ColString::new("panic!").set_color(BaseColor::RED),
        if let Some(l) = location {
            format!(
                "In {} - {}{}{}",
                ColString::new(l.file()).set_color(BaseColor::YELLOW),
                ColString::new(l.line()).set_color(BaseColor::YELLOW),
                ColString::new(":").set_color(BaseColor::YELLOW),
                ColString::new(l.column()).set_color(BaseColor::YELLOW),
            )
        } else {
            "".to_string()
        },
        if !content.is_empty() {
            format!("\n\t---\n\t{}", content)
        } else {
            "".to_string()
        }
    );
}
