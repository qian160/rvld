#![allow(non_snake_case)]
#![deny(unused)]
mod utils;
mod linker;

use linker::checkMagic;
use std::{env, fs};
use crate::utils::print;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() >= 2, "{}", color_text!("need an argument", 31));

    let file = &args[1];
    if let Ok(contents) = fs::read(file)
    {
        assert!(checkMagic(&contents), "{}", color_text!("not an ELF file!", 91));
    }
    else{
        error!("{}: not found!", file);
    }

    let file = linker::file::newFile(file);
    let inputFile = linker::inputfile::NewInputFile(file);
    println!("#sections = {}", inputFile.ElfSections.len());
}