// pasm - src/cli.rs
// -----------------
// made by matissoss
// licensed under MPL 2.0

use std::{env, sync::LazyLock};

pub static CLI: LazyLock<Cli> = LazyLock::new(|| Cli::new(env::args().collect::<Vec<String>>()));

use crate::shr::booltable::BoolTable16 as Flags;
use std::path::PathBuf;

const HELP: u8 = 0x0;
const DBG: u8 = 0x1;
const NOCOL: u8 = 0x2;
const SUPPORTED_INS: u8 = 0x3;
const SUPPORTEDINSR: u8 = 0x4;
const VER: u8 = 0x5;
const NO_ASSEMBLE: u8 = 0x6;
const NO_CHECK: u8 = 0x7;

#[derive(Default)]
pub struct Cli {
    target: Option<String>,   // -f flag
    infile: Option<PathBuf>,  // -i flag
    outfile: Option<PathBuf>, // -o flag
    flags: Flags,             // -/--flag
}

impl Cli {
    pub fn infile(&self) -> &Option<PathBuf> {
        &self.infile
    }
    pub fn outfile(&self) -> &Option<PathBuf> {
        &self.outfile
    }
    pub fn nocolor(&self) -> bool {
        self.flags.get(NOCOL).unwrap()
    }
    pub fn nocheck(&self) -> bool {
        self.flags.get(NO_CHECK).unwrap()
    }
    pub fn help(&self) -> bool {
        self.flags.get(HELP).unwrap()
    }
    pub fn version(&self) -> bool {
        self.flags.get(VER).unwrap()
    }
    pub fn debug(&self) -> bool {
        self.flags.get(DBG).unwrap()
    }
    pub fn check(&self) -> bool {
        self.flags.get(NO_ASSEMBLE).unwrap()
    }
    pub fn supported_instructions_raw(&self) -> bool {
        self.flags.get(SUPPORTEDINSR).unwrap()
    }
    pub fn supported_instructions(&self) -> bool {
        self.flags.get(SUPPORTED_INS).unwrap()
    }
    pub fn new(args: Vec<String>) -> Self {
        let mut cli = Self::default();
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
                "-C" | "--no-check" => cli.flags.set(NO_CHECK, true),
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
}
