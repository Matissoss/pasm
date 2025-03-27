//  rasmx86_64   -    help.rs
//  -------------------------
//  made by matissoss
//  licensed under MPL 2.0

#[allow(unused)]
static MAIN_HELP : &str = include_str!("hlp/main.txt");

#[allow(unused)]
pub struct Help;

#[allow(unused)]
impl Help {
    pub fn main_help(){
        print!("{}", MAIN_HELP);
    }
}
