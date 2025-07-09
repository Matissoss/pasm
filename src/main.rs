// pasm - src/main.rs
// ------------------
// made by matissoss
// licensed under MPL 2.0

#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::missing_transmute_annotations)]
#![allow(clippy::type_complexity)]
#![allow(clippy::to_string_trait_impl)]
#![allow(clippy::while_let_on_iterator)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::unnecessary_unwrap)]
// sometimes manual idx = 0; while idx < range { idx += 1 } is faster
// than Rust's preferred .iter().enumerate()
#![allow(clippy::explicit_counter_loop)]
#![allow(clippy::borrowed_box)]

// global imports go here
use std::process;

// local imports go here

// pasm modules
pub mod core;
pub mod libp;
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

use cli::*;

// feature dependent
#[cfg(feature = "time")]
use std::time;

// start
fn main() {
    switch_panichandler();
    let cli = &*CLI;

    if cli.version() {
        println!("{}", help::version());
        process::exit(0)
    }
    if cli.help() {
        println!("{}", help::help());
        process::exit(0)
    }
    #[cfg(feature = "iinfo")]
    if cli.supported_instructions() {
        print_supported_instructions();
        return;
    }
    #[cfg(feature = "iinfo")]
    if cli.supported_instructions_raw() {
        print_supported_instructions_raw();
        return;
    }

    let infile = if let Some(s) = cli.infile() {
        s
    } else {
        eprintln!("error: you forgot to provide -i=[PATH] flag");
        std::process::exit(1);
    };

    #[cfg(feature = "time")]
    let start = time::SystemTime::now();

    let file = match libp::get_file(infile.to_path_buf()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let ast = libp::pasm_parse_src(infile.to_path_buf(), &file, cli.nocheck());

    if let Err(errs) = ast {
        for mut e in errs {
            e.set_file(infile.to_path_buf());
            eprintln!("{e}");
        }
        std::process::exit(1);
    }
    let (ast, ptrs) = ast.unwrap();

    if cli.check() {
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

    let outfile = cli.outfile().clone();

    if let Err(e) = libp::assemble(ast, outfile) {
        eprintln!("{e}");
        std::process::exit(1);
    };
    for p in ptrs {
        unsafe { std::ptr::drop_in_place(p) };
    }
    #[cfg(all(feature = "time", feature = "vtime"))]
    utils::vtimed_print("overall", start);
    #[cfg(all(feature = "time", not(feature = "vtime")))]
    utils::vtimed_print(&format!("Assembling {}", infile.to_string_lossy()), start);
    process::exit(0);
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
