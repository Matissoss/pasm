// pasm - src/libp.rs
// ------------------
// made by matissoss
// licensed under MPL 2.0

use std::{error::Error, fs::OpenOptions, io::Read, path::Path};

use crate::{
    core::{
        api::{self, AssembleResult}, comp
    }, pre::{
        chk, par::{par, LineResult, ParserStatus}
    }, shr::{
        error::Error as PasmError,
        reloc::{RelType, Relocation},
    }, utils::LineIter
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
    let mut obuf: Vec<u8> = Vec::new();
    let mut rels: Vec<Relocation> = Vec::new();
    let mut line_iter = LineIter::new(&ibuf);
    let mut parser_status = ParserStatus::default();
    let bits = 16;
    while let Some((lnum, line)) = line_iter.next() {
        let line = line.trim();
        match par(&mut parser_status, line) {
            LineResult::Error(mut e) => {
                e.set_line(lnum);
                return Err(e);
            }
            LineResult::Instruction(i) => {
                let e = if bits == 64 {
                    chk::check_ins64bit(i)
                } else {
                    chk::check_ins32bit(i)
                };
                if let Err(e) = e {
                    e.set_line(lnum);
                    return Err(e);
                }
                let (res, mut rel_a) = comp::get_genapi(i, bits)
                    .assemble(i, bits, RelType::REL32);
                for r in rel_a.iter_mut() {
                    r.offset += obuf.len();
                }
                match res {
                    AssembleResult::WLargeImm(d) => obuf.extend(d.into_iter()),
                    AssembleResult::NoLargeImm(d) => obuf.extend(d),
                }
                rels.extend(rel_a.into_iter());
            }
            LineResult::Section(s) => {
                todo!()
            }
            LineResult::Label(l) => {
                todo!()
            }
            _ => {}
        }
    }

    match parser_status.attributes.get("target").unwrap_or("elf64") {
        #[cfg(feature = "target_elf")]
        "elf64" | "ELF64" => {

        },
        #[cfg(feature = "target_elf")]
        "elf32" | "ELF32" => {

        },
        "bin" => {

        }
        t => {
            return Err(PasmError::new(format!("unknown target {t}"), todo!()))
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
