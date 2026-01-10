// pasm - src/main.rs
// ------------------
// made by matissoss
// licensed under MPL 2.0

#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::missing_transmute_annotations)]
#![allow(clippy::to_string_trait_impl)]
#![allow(clippy::while_let_on_iterator)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::missing_safety_doc)]
// sometimes manual idx = 0; while idx < range { idx += 1 } is faster
// than Rust's preferred .iter().enumerate()
#![allow(clippy::explicit_counter_loop)]

// global imports go here
use std::process;

// local imports go here

// pasm modules
pub mod core;
#[cfg(not(feature = "refresh"))]
pub mod libp;
pub mod obj;
pub mod pre;
pub mod shr;

// pasm helper utilities
pub mod cli;
pub mod color;
pub mod conf;
pub mod consts;
pub mod help;
pub mod utils;

pub use shr::rpanic::switch_panichandler;

use cli::*;

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

    #[cfg(not(feature = "refresh"))]
    {
        let ipath = if let Some(ipath) = cli.infile() {
            ipath
        } else {
            eprintln!("You did not provide input file for pasm");
            process::exit(1);
        };
        let opath = if let Some(opath) = cli.outfile() {
            opath
        } else {
            eprintln!("You did not provide output file for pasm");
            process::exit(1);
        };
        if let Err(e) = libp::assemble(ipath, opath) {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}

#[cfg(feature = "iinfo")]
fn print_supported_instructions() {
    use crate::shr::mnemonic::Mnemonic;
    let ins_count = Mnemonic::__LAST as u16;
    println!("This version of PASM supports {} mnemonics!", ins_count - 1);
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
    println!()
}
#[cfg(feature = "iinfo")]
fn print_supported_instructions_raw() {
    use crate::shr::mnemonic::Mnemonic;
    for idx in 0..Mnemonic::__LAST as u16 {
        if idx + 1 == Mnemonic::__LAST as u16 {
            print!(
                "{}",
                unsafe { std::mem::transmute::<u16, Mnemonic>(idx) }.to_string()
            );
        } else {
            println!(
                "{}",
                unsafe { std::mem::transmute::<u16, Mnemonic>(idx) }.to_string()
            );
        }
    }
}
