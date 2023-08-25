use super::common::*;
use super::elf::ElfGetName;
use super::abi::*;
use super::file::{InputFile, File};
use super::elf::{Shdr, CheckFileCompatibility, Sym};
use super::sections::{InputSection, MergeableSection, MergedSection, SplitSection};
use super::symbol::Symbol;

#[derive(Default,Debug)]
pub struct Objectfile {
    pub inputFile:          Box<InputFile>,
    pub SymTabSec:          Option<Box<Shdr>>,
	pub SymtabShndxSec:     Vec<u32>,
	pub Sections:           Vec<Option<Rc<RefCell<InputSection>>>>,
    pub MergeableSections:  Vec<Option<MergeableSection>>
    // todo: handle shndx = SHN_COMMON?
    //pub Commons:        Rc<RefCell<InputSection>> ?
}

impl Deref for Objectfile {
    type Target = Box<InputFile>;

    fn deref(&self) -> &Self::Target {
        &self.inputFile
    }
}

impl DerefMut for Objectfile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inputFile
    }
}

impl Objectfile {
    pub fn new(ctx: &mut Context, file: Rc<File>, Alive: bool) -> Rc<RefCell<Self>> {
        CheckFileCompatibility(ctx, file.as_ref());
        let obj = Rc::new(RefCell::new(Objectfile {
            inputFile: InputFile::new(file), 
            ..Default::default()
        }));
        obj.borrow_mut().IsAlive = Alive;
        Objectfile::Parse(obj.clone(), ctx);
        return obj;
    }

    pub fn Name(&self) -> &String {
        &self.inputFile.Name//.clone()
    }

    pub fn Parse(obj: Rc<RefCell<Objectfile>>, ctx: &mut Context) {
        let mut o = obj.borrow_mut();

        o.SymTabSec = o.FindSection(SHT_SYMTAB);

        if let Some(symtab) = o.SymTabSec.clone() {
            o.FirstGlobal = symtab.Info as usize;
            o.FillUpElfSyms(&symtab);
            o.SymbolStrTab = o.GetBytesFromIdx(symtab.Link as usize);
        }

        drop(o);
        Objectfile::InitSections(obj.clone());
        Objectfile::InitSymbols(obj.clone(), ctx);
        Objectfile::InitMergeableSections(obj, ctx);
    }

    fn InitSections(obj: Rc<RefCell<Self>>) {
        let len = obj.borrow().ElfSections.len();
        obj.borrow_mut().Sections = vec![Default::default(); len];
        for i in 0..len {
            let shdr = unsafe{
                std::ptr::addr_of!(obj.borrow().ElfSections[i]).as_ref().unwrap()
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
                    obj.borrow_mut().Sections[i] = Some(sec);
                },
            }
        }
    }

    // find out which sections are mergeable
    fn InitMergeableSections(obj: Rc<RefCell<Self>>, ctx: &mut Context) {
        let mut o = obj.borrow_mut();
        let len = o.Sections.len();
        o.MergeableSections = vec![Default::default(); len];
        drop(o);
        for i in 0..len {
            let isec = obj.borrow().Sections[i].clone();
            if let Some(isec) = isec {
                let mut isecbm = isec.borrow_mut();
                if isecbm.IsAlive && isecbm.Shdr().Flags & SHF_MERGE != 0 {
                    isecbm.IsAlive = false;
                    drop(isecbm);
                    let ms = SplitSection(ctx, isec.clone());
                    obj.borrow_mut().MergeableSections[i] = Some(*ms);
                }
            }
        }
    }

    fn FillUpSymtabShndxSec(&mut self, shdr: &Shdr) {
        let bytes = InputFile::GetBytesFromShdr(&self, shdr);
        self.SymtabShndxSec = ReadSlice::<u32>(&bytes);
    }

    fn InitSymbols(file: Rc<RefCell<Self>>, ctx: &mut Context) {
        let mut obj = file.borrow_mut();
        if obj.SymTabSec.is_none(){
            return;
        }

        let n_locals = obj.FirstGlobal as usize;

        // first symbol is special, but here we won't deal with it now
        let firstSym = Symbol::new("");
        obj.LocalSymbols.push(firstSym.clone());
        obj.LocalSymbols[0].borrow_mut().File = Some(file.clone());
        obj.Symbols.insert(0, firstSym);

        // constract file.symbols from esyms
        for i in 1..n_locals {
            let esym = &obj.ElfSyms[i];
            let name = ElfGetName(&obj.SymbolStrTab, esym.Name as usize);
            let s = Symbol::new(&name);
            let mut sym = s.borrow_mut();
            sym.File = Some(file.clone());
            sym.Value = esym.Val;
            sym.SymIdx = i;
            if esym.IsAbs() == false {
                let isec = obj.Sections[obj.GetShndx(esym, i)].clone();
                sym.SetInputSection(isec);
            }
            obj.Symbols.insert(i, s.clone());
        }

        let globals = n_locals..obj.ElfSyms.len();
        for i in globals {
            let esym = &obj.ElfSyms[i];
            let name = ElfGetName(&obj.SymbolStrTab, esym.Name as usize);
            obj.Symbols.insert(i, Symbol::GetSymbolByName(ctx, &name));
        }
    }

    /// 1. esym.Shndx, (if Shndx is a normal value)
    /// 2. ShndxSec`[idx]` (Shndx == SHN_XINDEX)
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
        // local symbols dont need to resolve
        // the ith global symbol
        for i in obj.FirstGlobal..obj.ElfSyms.len() {
            let mut sym = obj.Symbols.get(&i).unwrap().borrow_mut();
            let esym = &obj.ElfSyms[i];

            if esym.IsUndef() {
                // nothing we can do here, impossible to find out where that symbol comes from
                continue;
            }

            // common symbols do not have a particular input section
            // just skip here(temp solution)
            if esym.IsCommon() {
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
//                debug!("{}: defined by {}", &sym.Name, obj.Name());
                sym.File = Some(o.clone());
                sym.SetInputSection(isec);
                sym.Value = esym.Val;
                sym.SymIdx = i;
            }
        }
    }

    pub fn MarkLiveObjects(obj: &Rc<RefCell<Objectfile>>, ctx: &mut Context, roots: &mut Vec<Rc<RefCell<Objectfile>>>) {
        let obj = obj.borrow();

        assert!(obj.IsAlive);

        for i in obj.FirstGlobal..obj.ElfSyms.len() {
            let sym = obj.Symbols.get(&i).unwrap().borrow();
            let esym = &obj.ElfSyms[i];
            // note: we must arrange command line arguments in correct order.
            // objects that offer symbols must be put before whom use symbols
            if sym.File.is_none() {
//                debug!("\n{}: skipped", ElfGetName(&f.SymbolStrTab, esym.Name as usize));
                continue;
            }

            if esym.IsUndef() && !sym.FileAlive() {
                if let Some(file) = &sym.File {
                    file.borrow_mut().IsAlive = true;
//                    warn!("add alive '{}'", file.borrow().Name());
                    roots.push(file.clone());
                }
            }
        }
    }

    fn GetSection(&self, esym: &Sym, idx: usize) ->  Option<Rc<RefCell<InputSection>>> {
        self.Sections[self.GetShndx(esym, idx)].clone()
    }

    pub fn ClearSymbols(o: &Rc<RefCell<Objectfile>>) {
        let mut f = o.borrow_mut();

        for i in f.FirstGlobal..f.Symbols.len() {
            let sym = f.Symbols.get(&i).unwrap().borrow();
            if let Some(file) = &sym.File {
                if std::rc::Rc::ptr_eq(&o, &file) {
//                    warn!("{}: removed.", sym.Name);
                    drop(sym);
                    f.Symbols.remove(&i);
                }
            }
        }
    }

    // diff
    pub fn RegisterSectionPieces(obj: Rc<RefCell<Self>>) {
        let mut o = obj.borrow_mut();
        for m in &mut o.MergeableSections {
            if let Some(ms) = m {
                let len = ms.Strs.len();
                ms.Fragments = Vec::with_capacity(len);
                for i in 0..len {
                    ms.Fragments.push(
                        *MergedSection::Insert(ms.Parent.clone(), ms.Strs[i].clone(), ms.P2Align)
                    );
                }
            }
        }

        let len = o.ElfSyms.len();
        for i in 1..len {
            let sym = o.Symbols.get(&i).unwrap();
            let esym = &o.ElfSyms[i];

            if esym.IsAbs() || esym.IsUndef() || esym.IsCommon() {
                continue;
            }

            match &o.MergeableSections[o.GetShndx(esym, i)] {
                Some(m) => {
                    let (frag, offset) = m.GetFragment(esym.Val as u32);
                    if frag.is_none() {
                        error!("bad symbol value");
                    }
                    sym.borrow_mut().SetSectionFragment(frag);
                    sym.borrow_mut().Value = offset as u64;
                },
                None => continue
            };

        }

    }

    pub fn IsAlive(&self) -> bool {
        self.IsAlive
    }
}
