//! module about file and io

use super::elf::{Sym, FileType};
use super::elf::checkMagic;
use super::archive::ReadArchiveMembers;
use super::symbol::Symbol;
use super::common::*;

#[derive(Default, Clone,Debug)]
pub struct File {
    pub Name:           String,
    pub Contents:       Vec<u8>,
    pub Type:           FileType,
    // objects in archive file share the same parent
    pub Parent:         Option<Rc<File>>,
}

// context and inputfile both has symbols... so maybe use rc is better
#[derive(Default,Debug)]
pub struct InputFile {
    pub File:           Rc<File>,
    pub ElfSections:    Vec<Shdr>,
    /// for common symbols
	pub ElfSections2:   Vec<Shdr>,
    pub ElfSyms:        Vec<Rc<Sym>>,
    pub FirstGlobal:    usize,
    pub Shstrtab:       ByteSequence,
    pub SymbolStrTab:   ByteSequence,
    pub IsAlive:        bool,
    /// use `shndx` as the key
    pub Symbols:        BTreeMap<usize, Rc<RefCell<Symbol>>>,
    pub LocalSymbols:   Vec<Rc<RefCell<Symbol>>>,
}

impl Deref for InputFile {
    type Target = Rc<File>;

    fn deref(&self) -> &Rc<File> {
        &self.File
    }
}

impl File {
    /// the contents is optional. the key is to avoid copy
    pub fn new(name: &str, contents: Vec<u8>, parent: Option<Rc<File>>) -> Rc<Self> {
        let Contents = if contents.len() == 0 {
            std::fs::read(name).expect(&format!("{} read failed", name))
        }
        else {
            contents
        };
        let ft: FileType;
        if Contents.len() == 0 {
            ft = FileType::FileTypeEmpty;
        }
        else if checkMagic(&Contents) {
            ft = match Read::<u16>(&Contents[16..]) {
                abi::ET_REL => FileType::FileTypeObject,
            _ =>    FileType::FileTypeUnknown
            };
        }
        else if Contents.starts_with(super::archive::AR_IDENT) {
            ft = FileType::FileTypeArchive;
        }
        else{
            ft = FileType::FileTypeUnknown;
        }

        Rc::new(
            File{
                Name: name.into(),
                Contents,
                Parent: parent,
                Type: ft,
            }
        )
    }
}

impl InputFile {
    pub fn new(file: Rc<File>) -> Box<Self> {
        let name = &file.Name;
        if file.Contents.len() < EHDR_SIZE {
            error!("{}: bad size!", name);
        }

        if checkMagic(&file.Contents) == false {
            error!("{}: not an ELF file!", name);
        }

        let mut f = InputFile{
            File: file,
            ..default()
        };

        let ehdr: Ehdr = Read::<Ehdr>(&f.Contents);

        let contents = &f.File.Contents[ehdr.ShOff as usize.. ];
        let shdr = Read::<Shdr>(&contents);
        let link = shdr.Link;

        // if the number of section header is larger than or equal to SHN_LORSERVE,
        // ehdr.shnum holds the value zero and the real number of entries in the section
        // header table is held in the shdr.size of the first entry in section header table
        let numSections = match ehdr.ShNum {
            0 => shdr.Size, 
            _ =>ehdr.ShNum as usize
        };

        f.ElfSections = vec![shdr];
        // read shdr
        contents.chunks_exact(SHDR_SIZE).skip(1).take(numSections).for_each(
            |shdr| {
                f.ElfSections.push(Read::<Shdr>(shdr));
            }
        );

        let mut shstrndx = ehdr.ShStrndx as usize;
        // escape. index stored elsewhere
        if ehdr.ShStrndx == abi::SHN_XINDEX {
            shstrndx = link as usize;
        }
        let slice = f.GetBytesFromIdx(shstrndx);

        f.Shstrtab = ByteSequence::new(slice.as_ptr(), slice.len());

        Box::new(f)
    }

    pub fn FindSection(&self, ty: u32) -> *const Shdr {
        for shdr in self.ElfSections.iter() {
            if shdr.Type == ty {
                return &*shdr;
            }
        }
        std::ptr::null()
    }

    pub fn GetBytesFromShdr(&self, s: &Shdr) -> &[u8] {
        let end = (s.Offset + s.Size) as usize;
        let Contents = &self.File.Contents;
        if Contents.len() < end {
            error!("section header is out of range: {}", s.Offset);
        }
        &Contents[s.Offset as usize..end]
    }

    pub fn GetBytesFromIdx(&self, idx: usize) -> &[u8] {
        self.GetBytesFromShdr(&self.ElfSections[idx])
    }

    pub fn FillUpElfSyms(&mut self, symtab: &Shdr) {
        let bytes = self.GetBytesFromShdr(symtab);
        let syms = ReadSlice::<Sym>(&bytes);
        self.ElfSyms.extend(syms.into_iter().map(Rc::new));
    }

    pub fn GetEhdr(&self) -> Ehdr {
        Read::<Ehdr>(&self.Contents)
    }
}

/// collect all the objects into ctx.objs, from input *.o or inside archives
pub fn ReadInputFiles(ctx: &mut Context, remaining: Vec<String>) {
    for arg in remaining {
        if let Some(arg) = arg.strip_prefix("-l") {
            ReadFile(ctx, FindLibrary(ctx, arg).unwrap());
        }
        else {
            ReadFile(ctx, File::new(&arg, vec![], None));
        }
    }
}

pub fn ReadFile(ctx: &mut Context, file: Rc<File>) {
    match file.Type {
        FileType::FileTypeObject => {
            let obj = Objectfile::new(ctx, file, true);
            ctx.Objs.push(obj);
        },
        FileType::FileTypeArchive => {
            for child in ReadArchiveMembers(file) {
                assert!(child.Type == FileType::FileTypeObject);
                let obj = Objectfile::new(ctx, child, false);
                ctx.Objs.push(obj);
            }
        },
        _ => {
            error!("unknown file type!");
        }
    }
}

pub fn OpenLibrary(path: &str) -> Option<Rc<File>> {
	match std::fs::read(path) {
        Ok(Contents) =>
            Some(File::new(path, Contents, None)),
        Err(_) => None
	}
}

pub fn FindLibrary(ctx: &Context, name: &str) -> Option<Rc<File>> {
	for dir in &ctx.Args.LIbraryPaths {
		let stem = dir.to_owned() + "/lib" + name + ".a";
		let f = OpenLibrary(&stem);
		if f.is_some() {
			return f;
		}
	}
    error!("library not found");
	None
}
