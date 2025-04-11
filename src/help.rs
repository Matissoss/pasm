// rasmx86_64 - help.rs
// --------------------
// made by matissoss
// licensed under MPL
static MAIN_HELP : &str = include_str!("hlp/main.txt");

pub struct Help;
impl Help {
    pub fn main_help(){
        print!("{}", MAIN_HELP);
    }
}
