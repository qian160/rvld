use std::rc::Rc;
use crate::error;
use super::context::Context;
use super::elf::{Shdr, Ehdr, Sym, FileType, ReadArchiveMembers, FindLibrary};
use super::elf::{SHDR_SIZE, SYM_SIZE, EHDR_SIZE, checkMagic};
use crate::utils::Read;

#[derive(Default, Clone)]
pub struct File {
    pub Name:           String,
    pub Contents:       Vec<u8>,
    pub Type:           FileType,
    // objects in archive file share the same parent
    pub Parent:         Option<Rc<File>>,
}

#[derive(Default)]
pub struct InputFile {
    pub File:           Box<File>,
    pub ElfSections:    Vec<Shdr>,
    pub ElfSyms:        Vec<Sym>,
    pub FirstGlobal:    u64,
    pub Shstrtab:       Vec<u8>,
    pub SymbolStrTab:   Vec<u8>,
}

#[derive(Default)]
pub struct Objectfile {
    pub inputFile:  Box<InputFile>,
    pub SymTabSec:  Box<Shdr>,
}

impl File {
    pub fn new(name: &str, contents: Option<Vec<u8>>) -> Box<Self> {
        let Contents = if contents.is_none() {
            std::fs::read(name).expect(&format!("{} read failed", name))
        }
        else {
            contents.unwrap()
        };
        let ft: FileType;
        if Contents.len() == 0 {
            ft = FileType::FileTypeEmpty;
        }
        else if checkMagic(&Contents) {
            ft = match Read::<u16>(&Contents[16..]).unwrap() {
                elf::abi::ET_REL => 
                    FileType::FileTypeObject,
                _ =>
                    FileType::FileTypeUnknown
            };
        }
        else if Contents.starts_with(super::elf::AR_IDENT) {
            ft = FileType::FileTypeArchive;
        }
        else{
            ft = FileType::FileTypeUnknown;
        }

        Box::new(
            File{
                Name: name.to_string(),
                Contents,
                Parent: None,
                Type: ft,
            })
    }
}

impl InputFile {
    pub fn new(file: Box<File>) -> Box<Self> {
        let name = &file.Name;
        crate::debug!("{}", name);
        
        if file.Contents.len() < EHDR_SIZE {
            error!("{}: bad size!", name);
        }

        if checkMagic(&file.Contents) == false {
            error!("{}: not an ELF file!", name);
        }
        drop(name);
        let mut f = InputFile{
            File: file,
            ..Default::default()
        };

        let ehdr: Ehdr = Read::<Ehdr>(&f.File.Contents).unwrap();
        let mut contents = &f.File.Contents[ehdr.ShOff as usize.. ];
        let shdr = Read::<Shdr>(&contents).unwrap();
        let mut num_sections = ehdr.ShNum as u64;

        if num_sections == 0 {
            num_sections = shdr.Size;
        }

        let link = shdr.Link;
        f.ElfSections = vec![shdr].into();

        while num_sections > 1 {
            contents = &contents[SHDR_SIZE..];
            f.ElfSections.push(Read::<Shdr>(&contents).unwrap());
            num_sections = num_sections - 1;
        }

        let mut shstrndx = ehdr.ShStrndx as usize;
        if ehdr.ShStrndx == elf::abi::SHN_XINDEX {
            shstrndx = link as usize;
        }
        f.Shstrtab = f.GetBytesFromIdx(shstrndx);

        Box::new(f)
    }

    pub fn FindSection(&self, ty: u32) -> Option<Box<Shdr>> {
        for shdr in self.ElfSections.iter() {
            if shdr.Type == ty {
                return Some(Box::new((*shdr).clone()));
            }
        }
        None
    }

    fn GetBytesFromShdr(&self, s: &Shdr) -> Vec<u8> {
        let end = (s.Offset + s.Size) as usize;
        let Contents = &self.File.Contents;
        if Contents.len() < end {
            error!("section header is out of range: {}", s.Offset);
        }
        Contents[s.Offset as usize..end].into()
    }

    pub fn GetBytesFromIdx(&self, idx: usize) -> Vec<u8> {
        self.GetBytesFromShdr(&self.ElfSections[idx])
    }

    pub fn FillUpElfSyms(&mut self, s: &Shdr) {
        let mut bs = self.GetBytesFromShdr(s);
        let mut n = bs.len() / SYM_SIZE;
        self.ElfSyms = Vec::with_capacity(n).into();
        while n > 0 {
            self.ElfSyms.push(Read::<Sym>(&bs).unwrap());
            bs = bs[SYM_SIZE..].into();
            n = n - 1;
        }
    }
}

impl Objectfile {
    // create a new object file and do the parse
    pub fn new(file: Box<File>) -> Box<Self> {
        let mut obj = Box::new(Objectfile {
            inputFile: InputFile::new(file), 
            SymTabSec: Default::default()
        });
        obj.Parse();
        return obj;
    }

    fn Parse(&mut self) {
        let symtab = self.inputFile.FindSection(elf::abi::SHT_SYMTAB);
        if symtab.is_some() {
            let file = &mut self.inputFile;
            let symtab = symtab.as_ref().unwrap();
            file.FirstGlobal = symtab.Info as u64;
            file.FillUpElfSyms(&*symtab);
            file.SymbolStrTab = file.GetBytesFromIdx(symtab.Link as usize);
        }
    }
}

pub fn ReadInputFiles(ctx: &mut Context, remaining: Vec<String>) {
    for arg in remaining {
        if let Some(arg) = arg.strip_prefix("-l") {
            ReadFile(ctx, FindLibrary(ctx, arg).unwrap());
        }
        else {
            ReadFile(ctx, File::new(&arg, None));
        }
    }
}

pub fn ReadFile(ctx: &mut Context, file: Box<File>) {
    match file.Type {
        FileType::FileTypeObject => {
            ctx.Objs.push(Objectfile::new(file));
        },
        FileType::FileTypeArchive => {
            for child in ReadArchiveMembers(file.into()) {
                assert!(child.Type == FileType::FileTypeObject);
                ctx.Objs.push(Objectfile::new(child));
            }
        },
        _ => {
            error!("unknown file type!");
        }
    }
}