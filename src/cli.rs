//  rasmx86_64 - src/cli.rs
//  -----------------------
//  made by matissoss
//  licensed under MPL 2.0

use crate::color::{ColString, Color, Modifier};
use std::{env, process, sync::LazyLock};

pub static CLI: LazyLock<Cli> = LazyLock::new(|| Cli::new(env::args().collect::<Vec<String>>()));

const VERBOSE: (&str, &str) = ("--verbose", "-v");
const DEBUG: (&str, &str) = ("--debug", "-d");
const NOCOL: (&str, &str) = ("--nocolor", "-n");

pub struct Cli {
    pub args: Vec<String>,
    // additional flags
    pub debug: bool,
    pub verbose: bool,
    pub nocolor: bool,
}

impl Cli {
    pub fn new(args: Vec<String>) -> Self {
        let (mut debug, mut verbose, mut nocolor) = (false, false, false);
        let mut argset = Vec::new();
        for arg in args {
            if arg == DEBUG.0 || arg == DEBUG.1 {
                debug = true;
            } else if arg == VERBOSE.0 || arg == VERBOSE.1 {
                verbose = true;
            } else if arg == NOCOL.0 || arg == NOCOL.1 {
                nocolor = true
            }
            argset.push(arg.to_string());
        }
        Cli {
            args: argset,
            debug,
            verbose,
            nocolor,
        }
    }
    pub fn has_arg(&self, vl: &str) -> bool {
        for a in &self.args {
            if let Some((a, _)) = a.split_once('=') {
                if a == vl {
                    return true;
                }
            } else {
                if a == vl {
                    return true;
                }
            }
        }
        false
    }
    pub fn get_kv_arg(&self, vl: &str) -> Option<&str> {
        for a in &self.args {
            if let Some((a, v)) = a.split_once('=') {
                if a == vl {
                    return Some(v);
                }
            }
        }
        None
    }
    pub fn get_arg(&self, searched: &str) -> Option<&String> {
        self.args.iter().find(|s| *s == searched)
    }
    #[inline(always)]
    pub fn exit(&self, path: &str, function: &str, cause: &str, exit_code: i32) -> ! {
        eprintln!(
            "[{}{}{}] ({} {}): {}",
            ColString::new(path)
                .set_color(Color::PURPLE)
                .set_modf(Modifier::Bold),
            ColString::new(':')
                .set_color(Color::PURPLE)
                .set_modf(Modifier::Bold),
            ColString::new(function)
                .set_color(Color::PURPLE)
                .set_modf(Modifier::Bold),
            ColString::new("EXIT")
                .set_color(Color::RED)
                .set_modf(Modifier::Bold),
            ColString::new(exit_code)
                .set_color(Color::RED)
                .set_modf(Modifier::Bold),
            cause
        );
        process::exit(exit_code);
    }
}
