#![allow(non_snake_case)]
//#![deny(unused)]
#![allow(unreachable_code)]

mod utils;
mod linker;
mod debug;

use linker::context::Context;
use linker::elf::GetMachineType;
use linker::file::File;
use std::cell::RefCell;
use std::io::Write;

use crate::linker::elf::{MachineType, checkMagic};
use crate::linker::passes;

fn main() {
    //std::env::args().into_iter().for_each(|x| info!("{}", x));
    let mut ctx = Context::new();
    let remaining = parseArgs(&mut ctx);

    // -m parameter not specified, try to infer it from an input file 
    if ctx.Args.Emulation == MachineType::MachineTypeNone {
        for filename in &remaining {
            // options the we dont care about
            if filename.starts_with("-") {
                continue;
            }
            let file = File::new(&filename, None, None);
            ctx.Args.Emulation = GetMachineType(&file);
            if ctx.Args.Emulation != MachineType::MachineTypeNone {
                break;
            }
        }
    }
    if ctx.Args.Emulation != MachineType::MachineTypeRISCV64 {
        error!("unknown emulation type!");
    }

    linker::file::ReadInputFiles(&mut ctx, remaining);
    passes::CreateInternalFile(&mut ctx);   debug!("before: #objs = {}", ctx.Objs.len());
    passes::ResolveSymbols(&mut ctx);       debug!("after: #objs = {}", ctx.Objs.len());
    passes::RegisterSectionPieces(&mut ctx);
    passes::ConvertCommonSymbols(&mut ctx);
    passes::ComputeMergedSectionSizes(&mut ctx);
    passes::CreateSyntheticSections(&mut ctx);
    passes::BinSections(&mut ctx);
    let chunks = passes::CollectOutputSections(&mut ctx);
    ctx.Chunks.extend(chunks);

    debug!("#chunks = {}", ctx.Chunks.len());

    passes::ComputeSectionSizes(&mut ctx);
    passes::SortOutputSections(&mut ctx);

    let ctx_ptr = std::ptr::addr_of_mut!(ctx);
    // mark
    for i in 0..ctx.Chunks.len() {
        let c = unsafe {&mut *ctx.Chunks[i]};
        c.UpdateShdr(ctx_ptr);
    }

    let fileSz = passes::SetOutputSectionOffsets(&mut ctx);
    debug!("file size = {fileSz}");

    let mut f = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&ctx.Args.Output).unwrap();

    ctx.Buf = vec![0; fileSz];

    for i in 0..ctx.Chunks.len() {
        let c = unsafe { &mut *ctx.Chunks[i]};
        c.CopyBuf(ctx_ptr);
    }

    f.write_all(&ctx.Buf).unwrap();
    assert!(checkMagic(&ctx.Buf));
}

pub fn parseArgs(ctx: &mut Box<Context>) -> Vec<String> {
    // skip rvld
    let args: RefCell<Vec<String>> = RefCell::new(std::env::args().skip(1).collect());
    let arg: RefCell<String> = RefCell::new(String::new());

    // add a '-' prefix to the string
    let dashes = |name: &str| {
        match name.len() {
            1 => vec![String::from("-") + &name],
            _ => vec![String::from("-") + &name, String::from("--") +&name]
        }
    };

    // options that need arguments. this will consume both the option and its arg(if has)
    let readArg = |name: &str| -> bool {
        let mut args = args.borrow_mut();
        let mut arg = arg.borrow_mut();
        for opt in dashes(name) {
            if args[0] == opt {
                if args.len() == 1 {
                    error!("option -{}: argument missing", &name);
                }
                *arg = args[1].clone();
                *args = args[2..].into();
                return true;
            }

            let mut prefix = opt;
            if name.len() > 1 {
                prefix += "=";
            }
            if args[0].starts_with(&prefix) {
                *arg = args[0].clone()[prefix.len()..].into();
                *args = args[1..].into();
                return true;
            }
        }
        return false;
    };
    // consume only the option(if has)
    let readFlag =  |name: &str| -> bool{
        let mut args = args.borrow_mut();
        for opt in dashes(&name) {
            if args[0] == opt {
                // match and advance by one
                *args = args[1..].into();
                return true;
            }
        }
        // not match, do nothing
        return false;
    };

    let mut remaining = vec![];
    while args.borrow_mut().len() > 0 {
        if readFlag("help") {
            info!("usage: {} [options] file...", std::env::args().next().unwrap());
            std::process::exit(0);
        }

        if readArg("o") || readArg("output") {
            ctx.Args.Output = arg.borrow().clone();
        }
        else if readArg("m") {
            let arch = arg.borrow();
            if *arch == String::from("elf64lriscv") {
                ctx.Args.Emulation = MachineType::MachineTypeRISCV64;
            }
            else {
                error!("unknown -m argument: {}", arch);
            }
        }
        else if readArg("L") {
            ctx.Args.LIbraryPaths.push(arg.borrow().clone());
        }
        else if readArg("l") {
            remaining.push("-l".to_string() + &arg.borrow());
        }
        else if readArg("sysroot")
            || readArg("plugin")
            || readArg("plugin-opt")
            || readArg("hash-style")
            || readArg("build-id")
            || readArg("z")
            || readFlag("static") 
            || readFlag("s")
            || readFlag("no-relax")
            || readFlag("as-needed")
            || readFlag("start-group")
            || readFlag("end-group") { /*ignored */}
        else if readFlag("v") || readFlag("version"){
            let git_output = std::process::Command::new("git")
                .args(&["rev-list", "-1", "HEAD"]).output();
            match git_output {
                Ok(out) => {
                    let version = String::from_utf8_lossy(&out.stdout)[0..6].to_string();
                    println!("rvld 0.1.0-{}", version);
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
            std::process::exit(0);
        }
        else {
            let mut args = args.borrow_mut();
            if args[0].starts_with("-"){
                error!("unknown command line option: {}", args[0]);
            }
            remaining.push(args[0].clone());
            *args = args[1..].into();
        }
    }
    return remaining;
}