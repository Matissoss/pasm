// pasm - src/main.rs
// ------------------
// made by matissoss
// licensed under MPL 2.0

#![allow(clippy::collapsible_else_if)]
#![allow(clippy::manual_map)]
#![allow(clippy::to_string_trait_impl)]
#![allow(clippy::while_let_on_iterator)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::unusual_byte_groupings)]

//  global imports go here
use std::{path::PathBuf, process};

// local imports go here

// pasm modules
pub mod core;
pub mod libr;
pub mod obj;
pub mod pre;
pub mod pre_core;
pub mod shr;

use core::comp;

// pasm helper utilities
pub mod cli;
pub mod color;
pub mod conf;
pub mod help;
pub mod utils;

pub use shr::rpanic::switch_panichandler;

use cli::CLI;

use help::Help;

// feature dependent
#[cfg(feature = "time")]
use std::time;

use crate::conf::RString;
// start
fn main() {
    switch_panichandler();
    let cli = &*CLI;

    if cli.has_arg("-h") || cli.has_arg("--help") {
        Help::main_help();
        process::exit(0)
    }
    #[cfg(feature = "iinfo")]
    if cli.has_arg("supported-instructions") {
        print_supported_instructions();
        return;
    }
    #[cfg(feature = "iinfo")]
    if cli.has_arg("supported-instructions-raw") {
        print_supported_instructions_raw();
        return;
    }

    let infile: PathBuf = if let Some(path) = cli.get_kv_arg("-i") {
        PathBuf::from(path)
    } else {
        println!("error: you forgot to provide -i=[PATH] flag");
        std::process::exit(1);
    };

    #[cfg(feature = "time")]
    let start = time::SystemTime::now();

    let ast = libr::par_file(&infile);

    if let Err(errs) = ast {
        for mut e in errs {
            e.set_file(infile.to_path_buf());
            eprintln!("{e}");
        }
        std::process::exit(1);
    }
    let ast = ast.unwrap();

    if cli.has_arg("check") {
        #[cfg(feature = "time")]
        {
            let end = time::SystemTime::now();
            println!(
                "Checking {} took {:08.16}s and ended without errors!",
                infile.to_string_lossy(),
                match end.duration_since(start) {
                    Ok(t) => t.as_secs_f32(),
                    Err(e) => e.duration().as_secs_f32(),
                }
            )
        }
        return;
    }

    let outfile: PathBuf = if let Some(path) = cli.get_kv_arg("-o") {
        PathBuf::from(path)
    } else {
        println!("error: you forgot to provide -o=[PATH] flag");
        std::process::exit(1);
    };

    if let Some(form) = cli.get_kv_arg("-f") {
        if let Err(e) = libr::assemble(ast, &outfile, form) {
            eprintln!("{e}");
            std::process::exit(1);
        };
        #[cfg(all(feature = "time", feature = "vtime"))]
        {
            let end = time::SystemTime::now();
            println!(
                "overall took {:08.16}s",
                match end.duration_since(start) {
                    Ok(t) => t.as_secs_f32(),
                    Err(e) => e.duration().as_secs_f32(),
                }
            )
        }
        #[cfg(all(feature = "time", not(feature = "vtime")))]
        {
            let end = time::SystemTime::now();
            println!(
                "Assembling {} took {:08.16}s and ended without errors!",
                infile.to_string_lossy(),
                match end.duration_since(start) {
                    Ok(t) => t.as_secs_f32(),
                    Err(e) => e.duration().as_secs_f32(),
                }
            )
        }
    }
}

#[cfg(feature = "iinfo")]
fn print_supported_instructions() {
    use crate::shr::ins::Mnemonic;
    let ins_count = Mnemonic::__LAST as u16;
    println!("This version of RASM supports {} mnemonics!", ins_count - 1);
    println!("Here's a list of all of them:");
    for idx in (2..ins_count).step_by(3) {
        let ins2 = unsafe { std::mem::transmute::<u16, Mnemonic>(idx - 2).to_string() };
        let ins1 = unsafe { std::mem::transmute::<u16, Mnemonic>(idx - 1).to_string() };
        let ins0 = unsafe { std::mem::transmute::<u16, Mnemonic>(idx).to_string() };
        println!(
            "{ins2}{}{ins1}{}{ins0}",
            " ".repeat(conf::LINE_WIDTH - ins2.len()),
            " ".repeat(conf::LINE_WIDTH - ins1.len()),
        )
    }
    let mut mod_ = ins_count % 3;
    while mod_ != 0 {
        let ins0 = unsafe { std::mem::transmute::<u16, Mnemonic>(ins_count - mod_).to_string() };
        if mod_ - 1 == 0 {
            print!("{ins0}")
        } else {
            print!("{ins0}{}", " ".repeat(conf::LINE_WIDTH - ins0.len()));
        }
        mod_ -= 1;
    }
}
#[cfg(feature = "iinfo")]
fn print_supported_instructions_raw() {
    use crate::shr::ins::Mnemonic;
    for idx in 0..Mnemonic::__LAST as u16 {
        if idx + 1 == Mnemonic::__LAST as u16 {
            print!(
                "{}",
                format!("{:?}", unsafe { std::mem::transmute::<u16, Mnemonic>(idx) })
                    .to_lowercase()
            );
        } else {
            println!(
                "{}",
                format!("{:?}", unsafe { std::mem::transmute::<u16, Mnemonic>(idx) })
                    .to_lowercase()
            );
        }
    }
}
