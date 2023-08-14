use crate::debug::print;
use crate::error;
use crate::linker::file::File;
use crate::linker::elf::{Shdr, Ehdr, Sym};
use super::elf::{SHDR_SIZE, SYM_SIZE, EHDR_SIZE, checkMagic};
use crate::utils::Read;

use elf::abi::SHN_XINDEX;

#[derive(Default)]
pub struct InputFile {
    pub File:           Box<File>,
    pub Ehdr:           Box<Ehdr>,
    pub ElfSections:    Box<Vec<Shdr>>,
    pub ElfSyms:        Box<Vec<Sym>>,
    pub FirstGlobal:    u64,
    pub Shstrtab:       Box<Vec<u8>>,
    pub SymbolStrTab:   Box<Vec<u8>>,
}

impl InputFile {
    pub fn FindSection(&self, ty: u32) -> Option<Box<Shdr>> {
        for shdr in self.ElfSections.iter() {
            if shdr.Type == ty {
                return Some(Box::new((*shdr).clone()));
            }
        }
        None
    }

    pub fn GetBytesFromShdr(&self, s: &Shdr) -> Box<Vec<u8>> {
        let end = (s.Offset + s.Size) as usize;
        if self.File.Contents.len() < end {
            error!("section header is out of range: {}", s.Offset);
        }
        Box::new(self.File.Contents[s.Offset as usize..end].to_vec())
    }

    pub fn GetBytesFromIdx(&self, idx: usize) -> Box<Vec<u8>> {
        self.GetBytesFromShdr(&self.ElfSections[idx])
    }

    pub fn FillUpElfSyms(&mut self, s: &Shdr) {
        let mut bs = self.GetBytesFromShdr(s);
        let mut n = bs.len() / SYM_SIZE;
        self.ElfSyms = Vec::with_capacity(n).into();
        while n > 0 {
            self.ElfSyms.push(Read::<Sym>(&bs).unwrap());
            bs = Box::new(bs[SYM_SIZE..].to_vec());
            n = n - 1;
        }
    }
}

pub fn NewInputFile(file: Box<File>) -> Box<InputFile> {
    let name = file.Name.clone();
    if file.Contents.len() < EHDR_SIZE {
        error!("{}: bad size!", name);
    }
    let mut f = InputFile{
        File: file,
        ..Default::default()
    };

    if checkMagic(&f.File.Contents) == false {
        error!("{}: not an ELF file!", name);
    }

    let ehdr: Ehdr = Read::<Ehdr>(&f.File.Contents).unwrap();
    f.Ehdr = Box::new(ehdr.clone());

    let mut contents = f.File.Contents[ehdr.ShOff as usize.. ].to_vec();
    let shdr = Read::<Shdr>(&contents).unwrap();
    let mut num_sections = ehdr.ShNum as u64;

    if num_sections == 0 {
        num_sections = shdr.Size;
    }

    let link = shdr.Link;
    f.ElfSections = vec![shdr].into();

    while num_sections > 1 {
        contents = contents[SHDR_SIZE..].to_vec();
        f.ElfSections.push(Read::<Shdr>(&contents).unwrap());
        num_sections = num_sections - 1;
    }

    let mut shstrndx = ehdr.ShStrndx as usize;
    if ehdr.ShStrndx == SHN_XINDEX {
        shstrndx = link as usize;
    }
    f.Shstrtab = f.GetBytesFromIdx(shstrndx);

    Box::new(f)
}
