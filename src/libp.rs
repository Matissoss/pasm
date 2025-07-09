// pasm - src/libr.rs
// ------------------
// made by matissoss
// licensed under MPL 2.0

use std::{fs, io::Write, path::PathBuf};

use crate::*;

use pre::par::ParserStatus;

use shr::{
    ast::AST,
    error::Error,
    reloc,
    reloc::Relocation,
    section::{Section, SlimSection},
    symbol::{Symbol, SymbolType},
    visibility::Visibility,
};

pub fn get_file(inpath: PathBuf) -> Result<Vec<u8>, Error> {
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    let file = fs::read(&inpath);
    let pathstr = inpath.to_string_lossy();
    if file.is_err() {
        return Err(Error::new(
            format!("could not read a file named \"{pathstr}\""),
            13,
        ));
    }
    #[cfg(feature = "vtime")]
    utils::vtimed_print("read   ", start);
    Ok(file.unwrap())
}

pub fn pasm_parse_src(
    inpath: PathBuf,
    file: &[u8],
    nocheck: bool,
) -> Result<(AST, Vec<*mut Vec<u8>>), Vec<Error>> {
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    let mut lines = utils::LineIter::new(file);
    let mut errors = Vec::new();
    let mut par = ParserStatus {
        inroot: true,
        ..Default::default()
    };
    let mut ast = AST::default();
    while let Some((lnum, line)) = lines.next() {
        if line.is_empty() {
            continue;
        }
        let tok = pre::tok::tokl(line);
        if tok.is_empty() {
            continue;
        }
        match pre::mer::mer(tok, lnum) {
            Ok(m) => {
                let err = pre::par::par(&mut ast, m, &mut par);
                if !err.is_null() {
                    errors.push(unsafe { std::ptr::read(err) });
                }
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }

    if !par.label.name.is_empty() {
        par.section.content.push(par.label);
    }
    if par.section != Section::default() {
        ast.sections.push(par.section);
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    #[cfg(feature = "vtime")]
    utils::vtimed_print("pre    ", start);
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    //let mut ast = pre::par::par(mer)?;

    ast.validate().map_err(|e| vec![e])?;

    pre_core::post_process(&mut ast).map_err(|e| vec![e])?;

    #[cfg(feature = "vtime")]
    utils::vtimed_print("post   ", start);
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    if !nocheck {
        let res = pre::chk::check_ast(&ast);

        #[cfg(feature = "vtime")]
        utils::vtimed_print("chk    ", start);

        if let Some(errs) = res {
            let pathstr = inpath.to_string_lossy();
            for (lname, errs) in errs {
                println!("-- {pathstr}:{lname} --");
                for mut e in errs {
                    e.set_file(inpath.to_path_buf());
                    eprintln!("{e}");
                }
            }
            std::process::exit(1);
        }
    }
    let mut ptrs = Vec::new();
    if !ast.includes.is_empty() {
        for include in ast.includes.clone() {
            ptrs.push(std::ptr::from_mut(Box::leak(Box::new(
                get_file(include.clone()).map_err(|e| vec![e])?,
            ))));
            let (ast1, ptrs1) =
                pasm_parse_src(include, unsafe { &**ptrs.last().unwrap() }, nocheck)?;
            ast.extend(ast1).map_err(|e| vec![e])?;
            ptrs.extend(ptrs1)
        }
    }
    Ok((ast, ptrs))
}

pub fn assemble(ast: AST, opath: Option<PathBuf>) -> Result<(), Error> {
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();
    let opath = if let Some(p) = opath {
        p.to_path_buf()
    } else {
        ast.default_output
            .clone()
            .unwrap_or(std::path::PathBuf::from("a.out"))
    };

    let (mut wrt, rel, mut sym, slims) = {
        let mut wrt = Vec::new();
        let mut rel = Vec::new();
        let mut sym = Vec::new();
        let mut slims = Vec::with_capacity(ast.sections.len());

        let mut idx = 0;
        for s in &ast.sections {
            let soff = wrt.len();
            let (wrt_a, rel_a, sym_a) = process_section(s, idx, wrt.len());
            let ssz = wrt.len() - soff;
            wrt.extend(wrt_a);
            rel.extend(rel_a);
            sym.extend(sym_a);
            slims.push(SlimSection {
                name: s.name,
                align: s.align,
                attributes: s.attributes,
                offset: soff,
                size: ssz,
                bits: s.bits,
            });
            sym.push(Symbol {
                name: s.name,
                offset: soff,
                size: ssz,
                sindex: idx,
                stype: SymbolType::Section,
                visibility: Visibility::Local,
            });
            idx += 1;
        }
        (wrt, rel, sym, slims)
    };

    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&opath);
    if file.is_err() {
        return Err(Error::new(
            "failed to open output file with write permissions",
            13,
        ));
    }

    match ast.format.unwrap_or("bin").to_string().as_str() {
        "bin" => {
            reloc::relocate_addresses(&mut wrt, rel, &sym)?;
            write(&mut file.unwrap(), &wrt)?;
        }
        #[cfg(feature = "target_elf")]
        "elf32" => {
            sym.extend(comp::extern_trf(&ast.externs));
            let elf = crate::obj::Elf::new(&slims, &opath, &wrt, rel, &sym, false)?;
            wrt = elf.compile(false);
            write(&mut file.unwrap(), &wrt)?;
        }
        #[cfg(feature = "target_elf")]
        "elf64" => {
            sym.extend(comp::extern_trf(&ast.externs));
            let elf = crate::obj::Elf::new(&slims, &opath, &wrt, rel, &sym, true)?;
            wrt = elf.compile(true);
            write(&mut file.unwrap(), &wrt)?;
        }
        _ => {
            return Err(Error::new(
                "you tried to use unknown/unsupported format",
                13,
            ))
        }
    }
    #[cfg(feature = "vtime")]
    utils::vtimed_print("core   ", start);
    Ok(())
}

fn process_section<'a>(
    section: &'a Section<'a>,
    idx: u16,
    plen: usize,
) -> (Vec<u8>, Vec<Relocation<'a>>, Vec<Symbol<'a>>) {
    let mut wrt = Vec::new();
    let mut sym = Vec::new();
    let mut rel = Vec::new();

    for lbl in &section.content {
        let st = lbl.attributes.get_symbol_type();
        let vi = lbl.attributes.get_visibility();
        let nm = lbl.name;
        let (bts, mut rels) =
            comp::compile_label((&lbl.content, lbl.align, lbl.attributes.get_bits()), plen);
        for rel in &mut rels {
            rel.shidx = idx;
            rel.offset += wrt.len();
        }
        rel.extend(rels);

        sym.push(Symbol {
            stype: st,
            name: nm,
            visibility: vi,
            offset: wrt.len() - section.offset,
            size: bts.len(),
            sindex: idx,
        });
        wrt.extend(bts);
    }
    (wrt, rel, sym)
}

pub fn write(writer: &mut impl Write, con: &[u8]) -> Result<(), Error> {
    if writer.write_all(con).is_err() {
        return Err(Error::new("failed to write content to buffer", 13));
    }
    Ok(())
}
