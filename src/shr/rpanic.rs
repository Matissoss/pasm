// rasmx86_64 - src/shr/rpanic.rs
// ------------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::color::{
    ColString,
    BaseColor,
};

#[inline]
pub fn rpanic(src: &str, caller: &str, panic_code: i32, msg: &str) -> !{
    eprintln!("RASM [{} at {}] panicked with code: {}!\n\tcause: {}",
        ColString::new(src).set_color(BaseColor::RED), 
        ColString::new(caller).set_color(BaseColor::RED), 
        ColString::new(panic_code).set_color(BaseColor::YELLOW), 
        msg
    );
    std::process::abort()
}
