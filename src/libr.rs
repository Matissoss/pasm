// rasmx86_64 - src/libr.rs
// ------------------------
// made by matissoss
// licensed under MPL 2.0

use std::{fs, io::Write, path::Path};

use crate::*;

use shr::{ast::AST, error::RASMError as Error, reloc, symbol::*};

pub fn par_file(inpath: &Path) -> Result<AST, Vec<Error>> {
    let file = fs::read_to_string(inpath);
    let pathstr = inpath.to_string_lossy();
    if file.is_err() {
        return Err(vec![Error::msg(format!(
            "Couldn't open file \"{pathstr}\""
        ))]);
    }
    let file = file.unwrap();
    let mut toks = Vec::with_capacity(file.lines().count());
    for l in file.lines() {
        toks.push(pre::tok::Tokenizer::tokenize_line(l));
    }
    let mut ast = pre::par::Parser::build_tree(pre::lex::Lexer::parse_file(toks))?;

    pre_core::post_process(&mut ast).map_err(|e| vec![e])?;

    let res = pre::chk::check_ast(&ast);

    if let Some(errs) = res {
        for (lname, errs) in errs {
            println!("-- {pathstr}:{lname} --");
            for e in errs {
                shr::error::print_error(e, inpath)
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
            ast.extend(par_file(&include)?).map_err(|e| vec![e])?;
        }
    }
    ast.fix_entry();
    Ok(ast)
}

pub fn compile(ast: AST, opath: &Path, tgt: &str) -> Result<(), Error> {
    let wrt = com_file(ast, opath, tgt)?;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(opath);
    if let Err(e) = file {
        return Err(Error::msg(format!(
            "Failed to open file with write permissions:\n\t{e}"
        )));
    }
    write(&mut file.unwrap(), &wrt)?;
    Ok(())
}

fn com_file(mut ast: AST, opath: &Path, tgt: &str) -> Result<Vec<u8>, Error> {
    let mut wrt = Vec::new();
    let mut sym = Vec::new();
    let mut rel = Vec::new();
    for (idx, section) in ast.sections.iter_mut().enumerate() {
        let plen = wrt.len();
        section.offset = plen as u32;
        for lbl in &mut section.content {
            lbl.shidx = idx;
            let st = lbl.stype;
            let vi = lbl.visibility;
            let nm = lbl.name.clone();
            let (bts, mut rels) = comp::compile_label((&lbl.inst, lbl.align, lbl.bits), plen);
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
                is_extern: false,
            });
            wrt.extend(bts);
        }
        section.size = (wrt.len() - plen) as u32;
    }

    for (idx, section) in ast.sections.iter().enumerate() {
        sym.push(Symbol {
            name: section.name.clone(),
            offset: section.offset,
            size: section.size,
            sindex: idx as u16,
            visibility: Visibility::Local,
            stype: SymbolType::Section,
            is_extern: false,
        });
    }

    match tgt {
        "bin" => reloc::relocate_addresses(&mut wrt, rel, &sym)?,
        #[cfg(feature = "target_elf")]
        "elf32" | "elf64" => {
            sym.extend(comp::extern_trf(&ast.externs));
            let is_64bit = tgt == "elf64";
            let elf = crate::obj::Elf::new(&ast.sections, opath, &wrt, &rel, &sym, is_64bit)?;
            wrt = elf.compile(is_64bit);
        }
        _ => return Err(Error::msg(format!("Unknown format: \"{tgt}\""))),
    }

    Ok(wrt)
}
pub fn write(writer: &mut impl Write, con: &[u8]) -> Result<(), Error> {
    if let Err(e) = writer.write_all(con) {
        return Err(Error::msg(format!(
            "Failed to write content to buffer:\n\t{e}"
        )));
    }
    Ok(())
}
