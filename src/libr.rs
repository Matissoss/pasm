// pasm - src/libr.rs
// ------------------
// made by matissoss
// licensed under MPL 2.0

use std::{fs, io::Write, path::Path};

use crate::*;

use shr::{ast::AST, error::RError as Error, reloc, symbol::*, visibility::Visibility};

pub fn pasm_parse_src(inpath: &Path) -> Result<AST, Vec<Error>> {
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    let file = fs::read_to_string(inpath);
    let pathstr = inpath.to_string_lossy();
    if file.is_err() {
        return Err(vec![Error::new(
            format!("could not read a file named \"{pathstr}\""),
            13,
        )]);
    }
    #[cfg(feature = "vtime")]
    utils::vtimed_print("read   ", start);
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    let lines = utils::split_str_owned(&file.unwrap(), '\n');

    #[cfg(feature = "vtime")]
    utils::vtimed_print("split  ", start);
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    let lcount = lines.len();
    let mut toks = Vec::with_capacity(lcount);

    #[cfg(not(feature = "mthread"))]
    {
        let mut chars = Vec::new();
        for l in lines {
            toks.push(pre::tok::tokl(&mut chars, &l));
            chars.clear();
        }
    }

    #[cfg(feature = "mthread")]
    {
        let mut semaphore = crate::shr::semaphore::Semaphore::new(conf::THREAD_LIMIT as usize);
        let mut handles = Vec::with_capacity(lcount);
        let lcount = conf::TOK_LN_GROUP;
        let mut lns = Vec::with_capacity(lcount);
        for l in lines {
            if lns.len() < lcount {
                lns.push(l);
                continue;
            } else {
                semaphore.acquire();
            }
            handles.push(std::thread::spawn(move || {
                let mut toks = Vec::with_capacity(lcount);
                let mut chars = Vec::new();
                for l in lns {
                    toks.push(pre::tok::tokl(&mut chars, &l));
                    chars.clear();
                }
                toks
            }));
            lns = Vec::with_capacity(lcount);
            semaphore.release();
        }
        if !lns.is_empty() {
            handles.push(std::thread::spawn(move || {
                let mut chars = Vec::new();
                let mut toks = Vec::with_capacity(lcount);
                for l in lns {
                    toks.push(pre::tok::tokl(&mut chars, &l));
                    chars.clear();
                }
                toks
            }));
        }
        for handle in handles {
            let t = handle.join().expect("Failed to join a tokenizer handle");
            toks.extend(t);
        }
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

    //ast.validate().map_err(|e| vec![e])?;

    pre_core::post_process(&mut ast).map_err(|e| vec![e])?;

    #[cfg(feature = "vtime")]
    utils::vtimed_print("post   ", start);
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    let res = pre::chk::check_ast(&ast);

    #[cfg(feature = "vtime")]
    utils::vtimed_print("chk    ", start);

    if let Some(errs) = res {
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
        let pths = {
            let mut v = Vec::with_capacity(ast.includes.len());
            for i in &ast.includes {
                v.push(i.clone());
            }
            v
        };
        for include in pths {
            ast.extend(pasm_parse_src(&include)?).map_err(|e| vec![e])?;
        }
    }
    Ok(ast)
}

pub fn assemble(ast: AST, opath: Option<&Path>) -> Result<(), Error> {
    #[cfg(feature = "vtime")]
    let start = std::time::SystemTime::now();

    let opath = if let Some(p) = opath {
        p.to_path_buf()
    } else {
        ast.default_output
            .clone()
            .unwrap_or(std::path::PathBuf::from("a.out"))
    };

    let wrt = assemble_file(ast, &opath)?;
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

// TODO: reimplement multithreading
fn assemble_file(ast: AST, opath: &std::path::PathBuf) -> Result<Vec<u8>, Error> {
    let mut wrt = Vec::new();
    let mut sym = Vec::new();
    let mut rel = Vec::new();

    let mut sections = ast.sections;
    for (idx, section) in sections.iter_mut().enumerate() {
        let plen = wrt.len();
        section.offset = plen as u32;
        for lbl in &mut section.content {
            let st = lbl.attributes.get_symbol_type();
            let vi = lbl.attributes.get_visibility();
            let nm = lbl.name.clone();
            let (bts, mut rels) =
                comp::compile_label((&lbl.content, lbl.align, lbl.attributes.get_bits()), plen);
            for rel in &mut rels {
                rel.shidx = idx as u16;
                rel.offset += wrt.len() as u32;
            }
            rel.extend(rels);

            sym.push(Symbol {
                stype: st,
                name: nm,
                visibility: vi,
                offset: wrt.len() as u32 - section.offset,
                size: bts.len() as u32,
                sindex: idx as u16,
            });
            wrt.extend(bts);
        }
        section.size = (wrt.len() - plen) as u32;
        sym.push(Symbol {
            name: section.name.clone(),
            offset: section.offset,
            size: section.size,
            sindex: idx as u16,
            visibility: Visibility::Local,
            stype: SymbolType::Section,
        });
    }

    match ast
        .format
        .unwrap_or(RString::from("bin"))
        .to_string()
        .as_str()
    {
        "bin" => reloc::relocate_addresses(&mut wrt, rel, &sym)?,
        #[cfg(feature = "target_elf")]
        "elf32" => {
            sym.extend(comp::extern_trf(&ast.externs));
            let elf = crate::obj::Elf::new(&sections, &opath, &wrt, &rel, &sym, false)?;
            wrt = elf.compile(false);
        }
        #[cfg(feature = "target_elf")]
        "elf64" => {
            sym.extend(comp::extern_trf(&ast.externs));
            let elf = crate::obj::Elf::new(&sections, &opath, &wrt, &rel, &sym, true)?;
            wrt = elf.compile(true);
        }
        _ => {
            return Err(Error::new(
                "you tried to use unknown/unsupported format",
                13,
            ))
        }
    }
    drop(sym);
    Ok(wrt)
}

pub fn write(writer: &mut impl Write, con: &[u8]) -> Result<(), Error> {
    if writer.write_all(con).is_err() {
        return Err(Error::new("failed to write content to buffer", 13));
    }
    Ok(())
}
