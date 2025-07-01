// pasm - src/cli.rs
// -----------------
// made by matissoss
// licensed under MPL 2.0

use std::{env, sync::LazyLock};

pub static CLI: LazyLock<Cli> = LazyLock::new(|| cli_new(env::args().collect::<Vec<String>>()));

use crate::shr::booltable::BoolTable16 as Flags;
use std::path::PathBuf;

const HELP: u8 = 0x0;
const DBG: u8 = 0x1;
const NOCOL: u8 = 0x2;
const SUPPORTED_INS: u8 = 0x3;
const SUPPORTEDINSR: u8 = 0x4;
const VER: u8 = 0x5;
const NO_ASSEMBLE: u8 = 0x6;

#[derive(Default)]
pub struct Cli {
    target: Option<String>,   // -f flag
    infile: Option<PathBuf>,  // -i flag
    outfile: Option<PathBuf>, // -o flag
    flags: Flags,             // -/--flag
}

pub fn cli_infile(cli: &Cli) -> &Option<PathBuf> {
    &cli.infile
}
pub fn cli_outfile(cli: &Cli) -> &Option<PathBuf> {
    &cli.outfile
}
pub fn cli_nocolor(cli: &Cli) -> bool {
    cli.flags.get(NOCOL).unwrap()
}
pub fn cli_help(cli: &Cli) -> bool {
    cli.flags.get(HELP).unwrap()
}
pub fn cli_target(cli: &Cli) -> Option<&str> {
    cli.target.as_deref()
}
pub fn cli_version(cli: &Cli) -> bool {
    cli.flags.get(VER).unwrap()
}
pub fn cli_check(cli: &Cli) -> bool {
    cli.flags.get(NO_ASSEMBLE).unwrap()
}
pub fn cli_debug(cli: &Cli) -> bool {
    cli.flags.get(DBG).unwrap()
}
pub fn cli_supported_instructions_raw(cli: &Cli) -> bool {
    cli.flags.get(SUPPORTEDINSR).unwrap()
}
pub fn cli_supported_instructions(cli: &Cli) -> bool {
    cli.flags.get(SUPPORTED_INS).unwrap()
}

pub fn cli_new(args: Vec<String>) -> Cli {
    let mut cli = Cli::default();
    for a in &args[1..] {
        let (key, val) = match a.split_once('=') {
            Some((key, val)) => (key, Some(val)),
            None => (a.as_str(), None),
        };
        match key {
            "-h" | "--help" => cli.flags.set(HELP, true),
            "-d" | "--debug" => cli.flags.set(DBG, true),
            "-i" | "--input" => {
                cli.infile = val.map(|v| v.into());
            }
            "-o" | "--output" => {
                cli.outfile = val.map(|v| v.into());
            }
            "-f" => {
                cli.target = val.map(|v| v.into());
            }
            "-v" | "--version" => cli.flags.set(VER, true),
            "-c" | "--check" => cli.flags.set(NO_ASSEMBLE, true),
            "-s" | "--supported-instructions" => cli.flags.set(SUPPORTED_INS, true),
            "-S" | "--supported-instructions-raw" => cli.flags.set(SUPPORTEDINSR, true),
            "-n" | "--nocolor" => cli.flags.set(NOCOL, true),
            _ => continue,
        }
    }
    cli
}
