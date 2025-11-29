// pasm - src/libp.rs
// ------------------
// made by matissoss
// licensed under MPL 2.0

use std::{fs::OpenOptions, io::Read, path::Path};

use crate::{
    pre::par::{par, LineResult, ParserStatus},
    shr::error::Error as PasmError,
    utils::LineIter,
};

pub fn assemble(ipath: &Path, opath: &Path) -> Result<(), PasmError> {
    // fetch input file
    let ifile = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(ipath);
    let mut ifile = match ifile {
        Ok(f) => f,
        Err(e) => {
            return Err(PasmError::new(e.to_string(), 6));
        }
    };
    let mut ibuf = Vec::new();
    if let Err(e) = ifile.read_to_end(&mut ibuf) {
        return Err(PasmError::new(e.to_string(), 6));
    }

    // get output from assembler
    let _obuf: Vec<u8> = Vec::new();
    let mut line_iter = LineIter::new(&ibuf);
    let mut parser_status = ParserStatus::default();
    while let Some((lnum, line)) = line_iter.next() {
        let line = line.trim();
        match par(&mut parser_status, line) {
            LineResult::Error(mut e) => {
                e.set_line(lnum);
                return Err(e);
            }
            LineResult::Instruction(_i) => {
                todo!()
            }
            LineResult::Section(_s) => {
                todo!()
            }
            LineResult::Label(_l) => {
                todo!()
            }
            _ => {}
        }
    }

    // now write content to a file
    let ofile = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(opath);
    let _ofile = match ofile {
        Ok(f) => f,
        Err(e) => {
            return Err(PasmError::new(e.to_string(), 6));
        }
    };

    Ok(())
}
