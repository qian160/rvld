use crate::error;
use crate::linker::file::File;
use crate::linker::elf::{Shdr, Ehdr};
use super::elf::ShdrSize;
use super::{EhdrSize, checkMagic};
use crate::utils::{print, Read};

pub struct InputFile {
    pub File:           Box<File>,
    pub ElfSections:    Vec<Shdr>,
}

pub fn NewInputFile(file: Box<File>) -> Box<InputFile> {
    let name = file.Name.clone();
    if file.Contents.len() < EhdrSize {
        error!("{}: bad size!", name);
    }
    let mut f = InputFile{File: file, ElfSections: vec![]};

    if checkMagic(&f.File.Contents) == false {
        error!("{}: not an ELF file!", name);
    }

    let ehdr: Ehdr = Read::<Ehdr>(&f.File.Contents).unwrap();
    let mut contents = f.File.Contents[ehdr.ShOff as usize.. ].to_vec();
    let shdr = Read::<Shdr>(&contents).unwrap();
    let mut num_sections = ehdr.ShNum as u64;

    if num_sections == 0 {
        num_sections = shdr.Size;
    }
    f.ElfSections = vec![shdr];

    while num_sections > 1 {
        contents = contents[ShdrSize..].to_vec();
        f.ElfSections.push(Read::<Shdr>(&contents).unwrap());
        num_sections = num_sections - 1;
    }

    Box::new(f)
}
