//  rasmx86_64  -   cli.rs
//  ----------------------
//  made by matissoss
//  licensed under MPL 2.0

use std::{
    path::PathBuf,
    sync::LazyLock,
    collections::HashSet,
    env,
    process
};

pub static CLI : LazyLock<Cli> = LazyLock::new(|| {
    Cli::new(env::args().collect::<Vec<String>>())
});

const VERBOSE : (&str, &str) = ("--verbose"     , "-v");
const DEBUG   : (&str, &str) = ("--debug"       , "-d");

#[allow(unused)]
pub struct Cli{
    args    : HashSet<String>,
    // additional flags
    debug   : bool,
    verbose : bool,
}

#[allow(unused)]
impl Cli{
    pub fn new(args: Vec<String>) -> Self{
        let (mut debug, mut verbose) = (false,false);
        let mut argset = HashSet::new();
        for arg in &args{
            if arg == DEBUG.0 || arg == DEBUG.1{
                debug = true;
            }
            else if arg == VERBOSE.0 || arg == VERBOSE.1{
                verbose = true;
            }
            argset.insert(arg.to_string());
        }
        return Cli {
            args : argset,
            debug,
            verbose
        };
    }
    pub fn home_dir() -> Option<PathBuf>{
        if cfg!(windows) {return None;}
        #[allow(deprecated)]
        return env::home_dir();
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
    pub fn warn(&self, path: &str, function: &str, msg: &str){
        println!("[{}:{}] (WARN): {}", path, function, msg);
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
        println!("[{}:{}] (EXIT {}): {}", path, function, exit_code, cause);
        process::exit(exit_code);
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn arg_parsing_test(){
        let mut cli = Cli::new(["./executable", "--debug", "--verbose"].map(|s| s.to_string()).to_vec());
        assert!(cli.debug   == true);
        assert!(cli.verbose == true);
        cli = Cli::new(["./executable", "-d", "-v"].map(|s| s.to_string()).to_vec() );
        assert!(cli.debug   == true);
        assert!(cli.verbose == true);
    }
    #[test]
    fn key_value_args(){
        let cli = Cli::new(["./executable", "-i=file.asm", "-o=file.out"].map(|s| s.to_string()).to_vec());
        assert!(cli.get_arg("-i") == Some("file.asm"));
        assert!(cli.get_arg("-o") == Some("file.out"));
    }
}
