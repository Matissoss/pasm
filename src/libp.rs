// pasm - src/libr.rs
// ------------------
// made by matissoss
// licensed under MPL 2.0

use std::{fs, io::Write, path::PathBuf};

use crate::*;

use shr::{
    ast::AST,
    error::Error,
    reloc,
    reloc::Relocation,
    section::{Section, SlimSection},
    symbol::{Symbol, SymbolType},
    visibility::Visibility,
};

pub fn get_file(inpath: PathBuf) -> Result<String, Error> {
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    let file = fs::read_to_string(&inpath);
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

pub fn pasm_parse_src(inpath: PathBuf, file: &str) -> Result<AST, Vec<Error>> {
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    let lines = utils::split_str_ref(file.as_bytes(), '\n');

    #[cfg(feature = "vtime")]
    utils::vtimed_print("split  ", start);
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    let lcount = lines.len();
    let mut toks = Vec::with_capacity(lcount);

    for l in lines {
        toks.push(pre::tok::tokl(l));
    }
    #[cfg(feature = "vtime")]
    utils::vtimed_print("tok    ", start);
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    let mer = pre::mer::mer(toks);
    #[cfg(feature = "vtime")]
    utils::vtimed_print("mer    ", start);
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    let mut ast = pre::par::par(mer?)?;

    #[cfg(feature = "vtime")]
    utils::vtimed_print("par    ", start);
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    ast.validate().map_err(|e| vec![e])?;

    pre_core::post_process(&mut ast).map_err(|e| vec![e])?;

    #[cfg(feature = "vtime")]
    utils::vtimed_print("post   ", start);
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

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

    if !ast.includes.is_empty() {
        for include in ast.includes.clone() {
            // we have to Box::leak here, because file data lives through entirety of program
            // lifetime (aka 'static) :D
            //
            // if we'd want to free it, then we'd return Vec<*const String> alongside AST and free it later
            let file = Box::leak(Box::new(get_file(include.clone()).map_err(|e| vec![e])?));
            ast.extend(pasm_parse_src(include, file)?)
                .map_err(|e| vec![e])?;
        }
    }
    Ok(ast)
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
                offset: soff as u32,
                size: ssz as u32,
                bits: s.bits,
            });
            sym.push(Symbol {
                name: s.name,
                offset: soff as u32,
                size: ssz as u32,
                sindex: idx,
                stype: SymbolType::Section,
                visibility: Visibility::Local,
            });
            idx += 1;
        }
        (wrt, rel, sym, slims)
    };

    match ast.format.unwrap_or("bin").to_string().as_str() {
        "bin" => reloc::relocate_addresses(&mut wrt, rel, &sym)?,
        #[cfg(feature = "target_elf")]
        "elf32" => {
            sym.extend(comp::extern_trf(&ast.externs));
            let elf = crate::obj::Elf::new(&slims, &opath, &wrt, rel, &sym, false)?;
            wrt = elf.compile(false);
        }
        #[cfg(feature = "target_elf")]
        "elf64" => {
            sym.extend(comp::extern_trf(&ast.externs));
            let elf = crate::obj::Elf::new(&slims, &opath, &wrt, rel, &sym, true)?;
            wrt = elf.compile(true);
        }
        _ => {
            return Err(Error::new(
                "you tried to use unknown/unsupported format",
                13,
            ))
        }
    }

    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(opath);
    if file.is_err() {
        return Err(Error::new(
            "failed to open output file with write permissions",
            13,
        ));
    }

    #[cfg(feature = "vtime")]
    utils::vtimed_print("core   ", start);
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    write(&mut file.unwrap(), &wrt)?;

    #[cfg(feature = "vtime")]
    utils::vtimed_print("write  ", start);

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
            rel.offset += wrt.len() as u32;
        }
        rel.extend(rels);

        sym.push(Symbol {
            stype: st,
            name: nm,
            visibility: vi,
            offset: wrt.len() as u32 - section.offset,
            size: bts.len() as u32,
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
