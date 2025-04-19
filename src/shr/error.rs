// rasmx86_64 - error.rs
// ---------------------
// made by matissoss
// licensed under MPL

use crate::CLI;
use std::{
    fs::{
        File,
        OpenOptions
    },
    path::PathBuf,
    fmt::{
        Formatter,
        Display,
        Error,
    },
    io::Read,
    sync::LazyLock
};

static ERR_CTX: LazyLock<(File, PathBuf)> = LazyLock::new(|| {
    let path = PathBuf::from(CLI.get_arg("-i").unwrap());
    (OpenOptions::new()
        .write(false)
        .append(false)
        .truncate(false)
        .create_new(false)
        .create(false)
        .read(true)
        .open(&path)
        .unwrap(), path)
});

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
    etype   : ExceptionType,
    msg     : Option<String>,
    tip     : Option<String>,
}

impl Display for RASMError{
    fn fmt(&self, frm: &mut Formatter<'_>) -> Result<(), Error>{
        let ctx = {
            let mut buf = String::new();
            (&ERR_CTX.0).read_to_string(&mut buf).unwrap();
            buf.lines().map(|l| l.to_string()).collect::<Vec<String>>()[self.line.unwrap()-1].clone()
        };

        writeln!(frm,
            "{}:\n\t{} at line {}\n\t{}{}{}",
            self.etype,
            ColString::new(ERR_CTX.1.to_string_lossy()).set_color(BaseColor::YELLOW),
            ColString::new(self.line.unwrap()).set_color(BaseColor::YELLOW),
            ColString::new(ctx).set_color(BaseColor::GREEN).set_modf(Modifier::Bold),
            if let Some(msg) = &self.msg{
                format!("\n\t---\n\t{}", msg)
            } else {"".to_string()},
            if let Some(tip) = &self.tip {
                format!("\n\t{} {}",
                    ColString::new("tip:")
                        .set_color(BaseColor::BLUE)
                        .set_modf (Modifier::Bold),
                    ColString::new(tip)
                        .set_color(BaseColor::BLUE)
                        .set_modf (Modifier::Bold)
                )
            } else {"".to_string()}
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
    pub fn new(line: Option<usize>, etype: ExceptionType, msg: OS, tip: OS) -> Self{
        return Self{
            line,
            etype,
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
    pub fn set_line(&mut self, newline: usize){
        self.line = Some(newline);
    }
}
