use std::{rc::Rc, cell::RefCell};
use std::ops::Deref;

use crate::{debug, warn, error, info};
use crate::utils::Read;
use super::ElfGetName;
use super::context::Context;
use super::file::{InputFile, File};
use super::elf::{Shdr, CheckFileCompatibility, Sym};
use super::sections::InputSection;
use super::symbol::Symbol;

use elf::abi::*;

#[derive(Default,Debug)]
pub struct Objectfile {
    pub inputFile:      Rc<RefCell<InputFile>>,
    pub SymTabSec:      Option<Box<Shdr>>,
	pub SymtabShndxSec: Vec<u32>,
	pub Sections:       Vec<Rc<RefCell<InputSection>>>,
    // ?
    pub Commons:        Vec<Rc<RefCell<InputSection>>>,
}

impl Deref for Objectfile {
    type Target = Rc<RefCell<InputFile>>;

    fn deref(&self) -> &Self::Target {
        &self.inputFile
    }
}

impl Objectfile {
    pub fn new(ctx: &mut Context, file: Rc<File>, Alive: bool) -> Rc<RefCell<Self>> {
        CheckFileCompatibility(ctx, file.as_ref());
        let obj = Rc::new(RefCell::new(Objectfile {
            inputFile: InputFile::new(file), 
            ..Default::default()
        }));
        obj.borrow().borrow_mut().IsAlive = Alive;
        Objectfile::Parse(obj.clone(), ctx);
        return obj;
    }

    pub fn Name(&self) -> String {
        self.inputFile.borrow().Name.clone()
    }

    pub fn Parse(obj: Rc<RefCell<Objectfile>>, ctx: &mut Context) {
        let mut objfile = obj.borrow_mut();

        objfile.SymTabSec = {
            let f = objfile.inputFile.borrow();
            f.FindSection(SHT_SYMTAB)
        };

        if let Some(symtab) = &objfile.SymTabSec {
            let mut inputfile = objfile.inputFile.borrow_mut();
            inputfile.FirstGlobal = symtab.Info as usize;
            inputfile.FillUpElfSyms(&symtab);
            inputfile.SymbolStrTab = inputfile.GetBytesFromIdx(symtab.Link as usize);
        }

        drop(objfile);
        Objectfile::InitSections(obj.clone());
        Objectfile::InitSymbols(obj.clone(), ctx);
    }

    fn InitSections(obj: Rc<RefCell<Objectfile>>) {
        let len = obj.borrow().inputFile.borrow().ElfSections.len();
        // crate::debug!("{}: {}", obj.borrow().Name(), len);
        obj.borrow_mut().Sections = vec![Default::default(); len];
        for i in 0..len {
            let shdr = unsafe{
                std::ptr::addr_of!(obj.borrow().borrow().ElfSections[i]).as_ref().unwrap()
            };
            match shdr.Type {
                // these massages are only used during linkding,
                // no need to put them into output file
                SHT_GROUP | SHT_SYMTAB | SHT_STRTAB | SHT_REL | SHT_RELA | SHT_NULL => {
                    continue;
                },
                SHT_SYMTAB_SHNDX => {
                    obj.borrow_mut().FillUpSymtabShndxSec(shdr);
                },
                _ => {
                    let sec = InputSection::new(obj.clone(), i);
                    // error. we should follow the index, or use a btreemap?
                    //obj.borrow_mut().Sections.push(sec);
                    obj.borrow_mut().Sections[i] = sec;
                },
            }
        }
    }

    fn FillUpSymtabShndxSec(&mut self, shdr: &Shdr) {
        let bytes = InputFile::GetBytesFromShdr(&self.borrow(), shdr);
        let nums = bytes.len() / std::mem::size_of::<u32>();
        for i in 0..nums {
            self.SymtabShndxSec.push(Read::<u32>(&bytes[4*i..]).unwrap());
        }
    }

    fn InitSymbols(file: Rc<RefCell<Self>>, ctx: &mut Context) {
        let obj = file.borrow();
        if obj.SymTabSec.is_none(){
            return;
        }

        let mut inputfile = obj.borrow_mut();

        let n_locals = inputfile.FirstGlobal as usize;

        inputfile.LocalSymbols = vec![Symbol::new(""); n_locals];

        inputfile.LocalSymbols[0].borrow_mut().File = Some(file.clone());

        // first symbol is special, but just skip it now
        for i in 1..n_locals {
            let esym = &inputfile.ElfSyms[i];
            let mut sym = inputfile.LocalSymbols[i].borrow_mut();
            sym.Name = ElfGetName(&inputfile.SymbolStrTab, esym.Name as usize);
            sym.File = Some(file.clone());
            sym.Value = esym.Val;
            sym.SymIdx = i;
            if esym.IsAbs() == false {
                let isec = Some(obj.Sections[obj.GetShndx(esym, i)].clone());
                sym.SetInputSection(isec);
            }
        }

        for i in 0..n_locals {
            let sym = inputfile.LocalSymbols[i].clone();
            inputfile.Symbols.push(sym);
        }

        let globals = n_locals..inputfile.ElfSyms.len();
        for i in globals {
            let esym = &inputfile.ElfSyms[i];
            let name = ElfGetName(&inputfile.SymbolStrTab, esym.Name as usize);
            inputfile.Symbols.push(Symbol::GetSymbolByName(ctx, &name));
        }
    }

    pub fn GetShndx(&self, esym: &Sym, idx: usize) -> usize {
        //assert!(idx != usize::MAX && idx < self.borrow().ElfSyms.len());

        if esym.Shndx == SHN_XINDEX {
            self.SymtabShndxSec[idx as usize] as usize
        }
        else {
            esym.Shndx as usize
        }
    }

    /// try to find out where the symbols come from, or the owner of each symbol
    pub fn ResolveSymbols(o: &Rc<RefCell<Self>>) {
        let obj = o.borrow_mut();
        let inputfile = obj.borrow();
        // local symbols dont need to resolve
        for i in inputfile.FirstGlobal..inputfile.ElfSyms.len() {
            let mut sym = inputfile.Symbols[i].borrow_mut();
            let esym = &inputfile.ElfSyms[i];
            if esym.IsUndef() {
                // nothing we can do here, impossible to find out where that symbol comes from
                continue;
            }

            let mut isec = None;
            // absolute symbols dont have related sections
            if esym.IsAbs() == false {
                isec = obj.GetSection(esym, i);
                if isec.is_none() {
                    continue;
                }
            }

            // current esym is not undef, and file unknown. which means 
            // that the symbol is defined by current object file.
            if sym.File.is_none() {
                debug!("{}: defined by {}", &sym.Name, obj.Name());
                sym.File = Some(o.clone());
                sym.SetInputSection(isec);
                sym.Value = esym.Val;
                sym.SymIdx = i;
            }
        }
    }

    pub fn MarkLiveObjects(obj: &Rc<RefCell<Objectfile>>, ctx: &mut Context, roots: &mut Vec<Rc<RefCell<Objectfile>>>) {
        let obj = obj.borrow();
        let f = obj.borrow();

        assert!(f.IsAlive);

        for i in f.FirstGlobal..f.ElfSyms.len() {
            let sym = f.Symbols[i].borrow();
            let esym = &f.ElfSyms[i];
//            debug!("{}", sym.Name);
            // note: we must arrange command line arguments in correct order.
            // objects that offer symbols must be put before whom use symbols
            if sym.File.is_none() {
//                debug!("\n{}: skipped", ElfGetName(&f.SymbolStrTab, esym.Name as usize));
                continue;
            }

            if esym.IsUndef() && !sym.FileAlive() {
                if let Some(file) = &sym.File {
                    file.borrow().borrow_mut().IsAlive = true;
                    warn!("add alive '{}'", file.borrow().Name());
                    roots.push(file.clone());
                }
            }
        }
    }

    fn GetSection(&self, esym: &Sym, idx: usize) ->  Option<Rc<RefCell<InputSection>>> {
        if idx < self.Sections.len() {
            if self.GetShndx(esym, idx) == elf::abi::SHN_COMMON as usize{
                return None;
            }
            return Some(self.Sections[self.GetShndx(esym, idx)].clone());
        }
        None
    }

    pub fn ClearSymbols(o: &Rc<RefCell<Objectfile>>) {
        let obj = o.borrow();
        let f = obj.borrow();

        for i in f.FirstGlobal..f.Symbols.len() {
            let mut sym = f.Symbols[i].borrow_mut();
            if let Some(f) = &sym.File {
                if std::rc::Rc::ptr_eq(&o, &f) {
//                    warn!("{}: cleared.", sym.Name);
                    sym.Clear();
                }
            }
        }
    }

    pub fn IsAlive(&self) -> bool {
        self.borrow().IsAlive
    }
}
