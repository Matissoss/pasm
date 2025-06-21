// rasmx86_64 - src/libr.rs
// ------------------------
// made by matissoss
// licensed under MPL 2.0

use std::{fs, io::Write, path::Path};

use crate::*;

use shr::{ast::AST, error::RASMError as Error, reloc, symbol::*};

pub fn par_file(inpath: &Path) -> Result<AST, Vec<Error>> {
    #[cfg(feature = "vtimed")]
    let start = std::time::SystemTime::now();

    let file = fs::read_to_string(inpath);
    let pathstr = inpath.to_string_lossy();
    if file.is_err() {
        return Err(vec![Error::msg(format!(
            "Couldn't open file \"{pathstr}\""
        ))]);
    }
    #[cfg(feature = "vtimed")]
    {
        let end = std::time::SystemTime::now();
        println!(
            "read    took {:02.16}s",
            end.duration_since(start).unwrap_or_default().as_secs_f32()
        );
    }
    #[cfg(feature = "vtimed")]
    let start = std::time::SystemTime::now();
    let lines = split_str_into_vec(file.unwrap());
    #[cfg(feature = "vtimed")]
    {
        let end = std::time::SystemTime::now();
        println!(
            "split   took {:02.16}s",
            end.duration_since(start).unwrap_or_default().as_secs_f32()
        );
    }
    #[cfg(feature = "vtimed")]
    let start = std::time::SystemTime::now();
    let lcount = lines.len();
    let mut toks = Vec::with_capacity(lcount);
    #[cfg(not(feature = "mthread"))]
    for l in lines {
        toks.push(pre::tok::Tokenizer::tokenize_line(&l));
    }
    #[cfg(feature = "mthread")]
    {
        use std::sync::atomic::Ordering;
        let semaphore = std::sync::atomic::AtomicU8::new(conf::THREAD_LIMIT);
        let mut handles = Vec::with_capacity(lcount);
        let lcount = (conf::THREAD_LIMIT as usize) << conf::TOK_LN_GROUP;
        let mut lns = Vec::with_capacity(lcount);
        for l in lines {
            if lns.len() < lcount {
                lns.push(l);
                continue;
            } else {
                loop {
                    let crt = semaphore.load(Ordering::Acquire);
                    // we wait until we get a permission to create new thread
                    if crt > 0 {
                        if semaphore
                            .compare_exchange(crt, crt - 1, Ordering::Acquire, Ordering::Relaxed)
                            .is_ok()
                        {
                            break;
                        }
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(conf::RETRY_TIME_MS));
                    }
                }
            }
            handles.push(std::thread::spawn(move || {
                let mut toks = Vec::with_capacity(lcount);
                for l in lns {
                    toks.push(pre::tok::Tokenizer::tokenize_line(&l))
                }
                toks
            }));
            lns = Vec::with_capacity(lcount);
            semaphore.fetch_add(1, Ordering::Release);
        }
        if !lns.is_empty() {
            handles.push(std::thread::spawn(move || {
                let mut toks = Vec::with_capacity(lcount);
                for l in lns {
                    toks.push(pre::tok::Tokenizer::tokenize_line(&l))
                }
                toks
            }));
        }
        for handle in handles {
            let t = handle.join().expect("Failed to join a tokenizer handle");
            toks.extend(t);
        }
    }
    #[cfg(feature = "vtimed")]
    {
        let end = std::time::SystemTime::now();
        println!(
            "tok     took {:02.16}s",
            end.duration_since(start).unwrap_or_default().as_secs_f32()
        );
    }
    #[cfg(feature = "vtimed")]
    let start = std::time::SystemTime::now();

    let mut ast = pre::par::Parser::build_tree(pre::lex::Lexer::parse_file(toks))?;

    pre_core::post_process(&mut ast).map_err(|e| vec![e])?;

    #[cfg(feature = "vtimed")]
    {
        let end = std::time::SystemTime::now();
        println!(
            "par/lex took {:02.16}s",
            end.duration_since(start).unwrap_or_default().as_secs_f32()
        );
    }

    #[cfg(feature = "vtimed")]
    let start = std::time::SystemTime::now();

    let res = pre::chk::check_ast(&ast);
    #[cfg(feature = "vtimed")]
    {
        let end = std::time::SystemTime::now();
        println!(
            "chk     took {:02.16}s",
            end.duration_since(start).unwrap_or_default().as_secs_f32()
        );
    }

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

fn split_str_into_vec(str: String) -> Vec<String> {
    let mut strs = Vec::with_capacity(8);
    let mut buf = Vec::with_capacity(24);
    for b in str.as_bytes() {
        if b != &b'\n' {
            buf.push(*b);
        } else if b == &b' ' {
            continue;
        } else {
            strs.push(String::from_utf8(buf).expect("Code should be encoded in UTF-8"));
            buf = Vec::new();
        }
    }
    if !buf.is_empty() {
        strs.push(String::from_utf8(buf).expect("Code should be encoded in UTF-8"));
    }
    strs
}

pub fn compile(ast: AST, opath: &Path, tgt: &str) -> Result<(), Error> {
    #[cfg(feature = "vtimed")]
    let start = std::time::SystemTime::now();
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
    #[cfg(feature = "vtimed")]
    {
        let end = std::time::SystemTime::now();
        println!(
            "core    took {:02.16}s",
            end.duration_since(start).unwrap_or_default().as_secs_f32()
        );
    }
    #[cfg(feature = "vtimed")]
    let start = std::time::SystemTime::now();
    write(&mut file.unwrap(), &wrt)?;
    #[cfg(feature = "vtimed")]
    {
        let end = std::time::SystemTime::now();
        println!(
            "write   took {:02.16}s",
            end.duration_since(start).unwrap_or_default().as_secs_f32()
        );
    }
    Ok(())
}

fn com_file(ast: AST, opath: &Path, tgt: &str) -> Result<Vec<u8>, Error> {
    #[cfg(feature = "mthread")]
    use std::sync::Arc;
    #[cfg(feature = "mthread")]
    use std::thread;
    let mut wrt = Vec::new();
    let mut sym = Vec::new();
    let mut rel = Vec::new();

    #[cfg(feature = "mthread")]
    let sections = crate::conf::Shared::new(ast.sections);
    #[cfg(not(feature = "mthread"))]
    let mut sections = ast.sections;
    #[cfg(not(feature = "mthread"))]
    for (idx, section) in sections.iter_mut().enumerate() {
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
    #[cfg(feature = "mthread")]
    let semaphore = std::sync::atomic::AtomicU8::new(crate::conf::THREAD_LIMIT);
    #[cfg(feature = "mthread")]
    for (shidx, mut section) in (*sections).clone().into_iter().enumerate() {
        use std::sync::atomic::Ordering;
        section.offset = wrt.len() as u32;
        let mut handles = Vec::with_capacity(crate::conf::THREAD_LIMIT as usize);

        let s_offset = Arc::new(wrt.len() as u32);
        let mut lbls = Vec::with_capacity(conf::CORE_LB_GROUP);
        for lbl in section.content {
            if lbls.len() < conf::CORE_LB_GROUP {
                lbls.push(lbl);
                continue;
            }
            loop {
                let crt = semaphore.load(Ordering::Acquire);
                // we wait until we get a permission to create new thread
                if crt > 0 {
                    if semaphore
                        .compare_exchange(crt, crt - 1, Ordering::Acquire, Ordering::Relaxed)
                        .is_ok()
                    {
                        break;
                    }
                } else {
                    thread::sleep(std::time::Duration::from_millis(conf::RETRY_TIME_MS));
                }
            }

            let len = wrt.len();
            let soff = Arc::clone(&s_offset);
            handles.push(thread::spawn(move || {
                let (mut bts, mut rel, mut sym) = (Vec::new(), Vec::new(), Vec::new());
                for label in lbls {
                    let (bts_n, mut rel_n) = comp::compile_label((&label.inst, label.align, label.bits), 0);
                    for rel in &mut rel_n {
                        rel.offset += len as u32;
                        rel.shidx = shidx as u16;
                    }
                    sym.push(Symbol {
                            name: label.name.clone(),
                            offset: len as u32 - *soff,
                            size: bts.len() as u32,
                            sindex: shidx as u16,
                            visibility: label.visibility,
                            stype: label.stype,
                            is_extern: false,
                    });
                    rel.push(rel_n);
                    bts.extend(bts_n);
                }
                (sym, rel, bts)
            }));
            lbls = Vec::new();
            semaphore.fetch_add(1, Ordering::Release);
        }
        if !lbls.is_empty() {
            let len = wrt.len();
            let soff = Arc::clone(&s_offset);
            handles.push(thread::spawn(move || {
                let (mut bts, mut rel, mut sym) = (Vec::new(), Vec::new(), Vec::new());
                for label in lbls {
                    let (bts_n, mut rel_n) = comp::compile_label((&label.inst, label.align, label.bits), 0);
                    for rel in &mut rel_n {
                        rel.offset += len as u32;
                        rel.shidx = shidx as u16;
                    }
                    sym.push(Symbol {
                            name: label.name.clone(),
                            offset: len as u32 - *soff,
                            size: bts.len() as u32,
                            sindex: shidx as u16,
                            visibility: label.visibility,
                            stype: label.stype,
                            is_extern: false,
                    });
                    rel.push(rel_n);
                    bts.extend(bts_n);
                }
                (sym, rel, bts)
            }));
        }
        for handle in handles {
            let (s, r, w) = handle.join().expect("Failed to wait");
            wrt.extend(w);
            for r in r {
                rel.extend(r);
            }
            sym.extend(s);
        }
    }

    #[cfg(not(feature = "mthread"))]
    for (idx, section) in sections.iter().enumerate() {
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
    #[cfg(feature = "mthread")]
    for (idx, section) in sections.clone().iter().enumerate() {
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
            let elf = crate::obj::Elf::new(sections, opath, &wrt, &rel, &sym, is_64bit)?;
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
