//  rasmx86_64  -   cli.rs
//  ----------------------
//  made by matissoss
//  licensed under MPL 2.0

use std::{
    sync::LazyLock,
    env,
    process
};
use crate::color::{
    ColString,
    Modifier,
    BaseColor
};
pub static CLI : LazyLock<Cli> = LazyLock::new(|| {
    Cli::new(env::args().collect::<Vec<String>>())
});
const VERBOSE : (&str, &str) = ("--verbose"     , "-v");
const DEBUG   : (&str, &str) = ("--debug"       , "-d");
const NOCOL   : (&str, &str) = ("--nocolor"     , "-n");

pub struct Cli{
    pub args    : Vec<String>,
    // additional flags
    pub debug   : bool,
    pub verbose : bool,
    pub nocolor : bool,
}

impl Cli{
    pub fn new(args: Vec<String>) -> Self{
        let (mut debug, mut verbose, mut nocolor) = (false,false,false);
        let mut argset = Vec::new();
        for arg in args{
            if arg == DEBUG.0 || arg == DEBUG.1{
                debug = true;
            }
            else if arg == VERBOSE.0 || arg == VERBOSE.1{
                verbose = true;
            }
            else if arg == NOCOL.0 || arg == NOCOL.1{
                nocolor = true
            }
            argset.push(arg.to_string());
        }
        return Cli {
            args : argset,
            debug,
            verbose,
            nocolor
        };
    }
    pub fn get_arg(&self, searched: &str) -> Option<&str>{
        for arg in &self.args{
            if let Some((key, value)) = arg.split_once('='){
                if key == searched {
                    return Some(value);
                }
            }
            else {
                if arg == searched {
                    return Some(arg);
                }
            }
        }
        return None;
    }
    #[inline(always)]
    pub fn debug(&self, path: &str, function: &str, msg: &str){
        if self.debug{
            println!("[{}:{}] (DEBUG): {}", path, function, msg);
        }
    }
    #[inline(always)]
    pub fn error(&self, path: &str, function: &str, msg: &str){
        println!("[{}:{}] (ERROR): {}", path, function, msg);
    }
    #[inline(always)]
    pub fn verbose(&self, path: &str, function: &str, msg: &str){
        if self.verbose{
            println!("[{}:{}] (VERBOSE): {}", path, function, msg);
        }
    }
    #[inline(always)]
    pub fn exit(&self, path: &str, function: &str, cause: &str, exit_code: i32) -> !{
        println!("[{}{}{}] ({} {}): {}", 
            ColString::new(path)     .set_color(BaseColor::PURPLE).set_modf(Modifier::Bold), 
            ColString::new(':')      .set_color(BaseColor::PURPLE).set_modf(Modifier::Bold),
            ColString::new(function) .set_color(BaseColor::PURPLE).set_modf(Modifier::Bold), 
            ColString::new("EXIT")   .set_color(BaseColor::RED)   .set_modf(Modifier::Bold),
            ColString::new(exit_code).set_color(BaseColor::RED)   .set_modf(Modifier::Bold), 
            cause
        );
        process::exit(exit_code);
    }
}
