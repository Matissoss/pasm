// rasmx86_64 - error.rs
// ---------------------
// made by matissoss
// licensed under MPL

use std::fmt::{
    Formatter,
    Display,
    Error,
};
use crate::color::{
    ColString,
    BaseColor,
    Modifier
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExceptionType{
    Warn,
    Error,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RASMError{
    line    : Option<usize>,
    cont    : Option<String>,
    etype   : ExceptionType,
    tip     : Option<String>,
    msg     : Option<String>,
}

impl Display for RASMError{
    fn fmt(&self, frm: &mut Formatter<'_>) -> Result<(), Error>{
        writeln!(frm, 
            "{}:\n\tAt line {}:\n\t{}\n\t---\n\t{}!\n\t===\n\ttip: {}", 
            self.etype,
            if let Some(line) = self.line{
                ColString::new(line).set_modf(Modifier::Bold).set_color(BaseColor::YELLOW)
            }
            else{
                ColString::new("unknown")
            },
            if let Some(cont) = &self.cont {
                ColString::new(cont).set_modf(Modifier::Bold).set_color(BaseColor::GREEN)
            }
            else {
                ColString::new("unknown")
            },
            if let Some(msg) = &self.msg {
                msg.as_str()
            }
            else {
                "unknown"
            },
            if let Some(tip) = &self.tip {
                ColString::new(tip).set_color(BaseColor::BLUE)
            }
            else {
                ColString::new("unknown")
            },
        )
    }
}

impl Display for ExceptionType{
    fn fmt(&self, frm: &mut Formatter<'_>) -> Result<(), Error>{
        write!(frm, "{}", 
            if let Self::Warn = self{
                ColString::new("warn").set_color(BaseColor::YELLOW)
            }
            else if let Self::Error = self{
                ColString::new("error").set_color(BaseColor::RED)
            }
            else {
                ColString::new("info").set_color(BaseColor::BLUE)
            }
        )
    }
}

type OS = Option<String>;
impl RASMError{
    pub fn new(line: Option<usize>, etype: ExceptionType, cont: OS, msg: OS, tip: OS) -> Self{
        return Self{
            line,
            etype,
            cont,
            msg,
            tip
        }
    }
    pub fn get_type(&self) -> &ExceptionType{
        return &self.etype;
    }
    pub fn get_line(&self) -> Option<&usize>{
        return self.line.as_ref();
    }
    pub fn get_context(&self) -> Option<&String>{
        return self.cont.as_ref();
    }
    pub fn set_line(&mut self, newline: usize){
        self.line = Some(newline);
    }
    pub fn set_cont(&mut self, cont: String){
        self.cont = Some(cont);
    }
}
