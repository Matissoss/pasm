// pasm - src/help.rs
// ------------------
// made by matissoss
// licensed under MPL 2.0

use crate::conf::*;

pub fn version() -> String {
    String::from(VER)
}

pub fn help() -> String {
    let mut help_string = format!("{BIN} - {VER}\n");

    help_string.push_str("Flags:\n");
    help_string.push_str("\t--help / -h                       ; prints this message\n");
    help_string
        .push_str("\t-d / --debug                      ; prints hex dump of each instruction\n");
    help_string.push_str("\t-i=[PATH] / --input=[PATH]        ; specifies input file\n");
    help_string.push_str(
        "\t-o=[PATH] / --output=[PATH]       ; specifies output file, by default a.out\n",
    );
    help_string.push_str("\t-v / --version                    ; prints version\n");
    #[cfg(feature = "iinfo")]
    help_string.push_str("\t-s / --supported-instructions     ; prints supported instructions\n");
    #[cfg(feature = "iinfo")]
    help_string.push_str(
        "\t-S / --supported-instructions-raw ; prints supported instructions in raw format\n",
    );
    help_string
        .push_str("\t-C / --skip-check                 ; skips checking part to gain performance (up to 25%)\n");
    help_string
        .push_str("\t-n / --nocolor                    ; prints all colored text without color\n");

    help_string.push_str("made by matissoss\nlicensed under MPL 2.0");
    help_string.push_str("\nsource code: https://github.com/Matissoss/pasm");
    help_string
}
