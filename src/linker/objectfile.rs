use super::elf::Shdr;
use super::file::File;
use super::inputfile::{InputFile, NewInputFile};
pub struct Objectfile {
    pub inputFile:  Box<InputFile>,
    pub SymTabSec:  Box<Shdr>,
}

pub fn NewObjectFile(file: Box<File>) -> Box<Objectfile> {
    Box::new(Objectfile {
        inputFile: NewInputFile(file), 
        SymTabSec: Default::default()
    })
}

impl Objectfile {
    pub fn Parse(&mut self) {
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