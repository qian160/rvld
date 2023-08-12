#![allow(non_snake_case)]
//#![deny(unused)]
mod utils;
mod linker;
mod debug;

use linker::elf::checkMagic;
use std::env;
use debug::print;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() >= 2, "{}", color_text!("need an argument", 31));

    let file = linker::file::newFile(&args[1]);
    assert!(checkMagic(&file.Contents), "{}", color_text!("not an ELF file!", 91));

    let mut objectfile = linker::objectfile::NewObjectFile(file);
    objectfile.Parse();

    for shdr in objectfile.inputFile.ElfSections.iter() {
        info!("{:x?}", shdr);
    }

    for sym in objectfile.inputFile.ElfSyms.iter(){
        info!("{:?}", sym);
        info!("{}", linker::ElfGetName(&objectfile.inputFile.SymbolStrTab, sym.Name as usize));
    }
}