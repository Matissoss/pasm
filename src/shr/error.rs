// rasmx86_64 - error.rs
// ---------------------
// made by matissoss
// licensed under MPL

use std::fmt::{
    Formatter,
    Display,
    Error,
};

use crate::color::ColorText;

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
            "{}:\n\tAt line {}:\n\t{}\n\t---\n\t{}\n\t===\n\ttip: {}", 
            self.etype,
            if let Some(line) = self.line{
                line.to_string().as_str().yellow()
            }
            else{
                "unknown".yellow()
            },
            if let Some(cont) = &self.cont {
                cont.as_str().green()
            }
            else {
                "unknown".green()
            },
            if let Some(msg) = &self.msg {
                msg.as_str()
            }
            else {
                "unknown"
            },
            if let Some(tip) = &self.tip {
                tip.as_str()
            }
            else {
                "unknown"
            },
        )
    }
}

impl Display for ExceptionType{
    fn fmt(&self, frm: &mut Formatter<'_>) -> Result<(), Error>{
        write!(frm, "{}", 
            if let Self::Warn = self{
                "warn".yellow()
            }
            else if let Self::Error = self{
                "error".red()
            }
            else {
                "info".blue()
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
}
