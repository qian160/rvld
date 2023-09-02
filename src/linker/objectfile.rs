use super::common::*;
use elf::abi::*;
use super::file::{InputFile, File};
use super::elf::{CheckFileCompatibility, Sym, ElfGetName};
use super::inputsections::{InputSection, MergeableSection, SplitSection};
use super::symbol::Symbol;
use super::output::MergedSection;

#[derive(Debug)]
pub struct Objectfile {
    pub hasCommon:          bool,
    pub inputFile:          Box<InputFile>,
    pub SymTabSec:          *const Shdr,
	pub SymtabShndxSec:     Vec<u32>,
	pub Sections:           Vec<Option<Rc<RefCell<InputSection>>>>,
    pub MergeableSections:  Vec<Option<MergeableSection>>
}

impl Default for Objectfile {
    fn default() -> Self {
        Self { 
            hasCommon:  false,
            SymTabSec:  std::ptr::null(),
            inputFile:  Default::default(),
            Sections:   Default::default(),
            SymtabShndxSec: Default::default(),
            MergeableSections:  Default::default()
        }
    }
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
        let obj = Objectfile {
            inputFile: InputFile::new(file), 
            ..Default::default()
        }.ToRcRefcell();
        obj.borrow_mut().IsAlive = Alive;
        Objectfile::Parse(obj.clone(), ctx);
        obj
    }

    pub fn Name(&self) -> &String {
        &self.Name
    }

    pub fn Parse(obj: Rc<RefCell<Objectfile>>, ctx: &mut Context) {
        let mut o = obj.borrow_mut();
        o.SymTabSec = o.FindSection(SHT_SYMTAB);

        //if let Some(symtab) = o.SymTabSec.clone() {
        if !o.SymTabSec.is_null() {
            let symtab = unsafe {&*o.SymTabSec};
            o.FirstGlobal = symtab.Info as usize;
            o.FillUpElfSyms(&symtab);
            let slice =o.GetBytesFromIdx(symtab.Link as usize);
            o.SymbolStrTab = ByteSequence::new(slice.as_ptr(), slice.len());
        }

        drop(o);
        Objectfile::InitSections(&obj, ctx);
        Objectfile::InitSymbols(&obj, ctx);
        Objectfile::InitMergeableSections(obj.clone(), ctx);
    }

    fn InitSections(obj: &Rc<RefCell<Self>>, ctx: &mut Context) {
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
                    let name = ElfGetName(&obj.borrow().Shstrtab.GetSlice(), shdr.Name as usize);
                    let sec = InputSection::new(ctx, name, obj.clone(), i);
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
                if isecbm.IsAlive && isecbm.Shdr().Flags & SHF_MERGE as u64 != 0 {
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

    fn InitSymbols(file: &Rc<RefCell<Self>>, ctx: &mut Context) {
        let mut obj = file.borrow_mut();
        if obj.SymTabSec.is_null(){
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
            let name = ElfGetName(&obj.SymbolStrTab.GetSlice(), esym.Name as usize);
            if esym.IsCommon() {
                error!("{name}: common local symbol?");
            }

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
            if obj.ElfSyms[i].IsCommon() {
                obj.hasCommon = true;
            }
            let name = ElfGetName(&obj.SymbolStrTab.GetSlice(), obj.ElfSyms[i].Name as usize);
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
        // local symbols dont need to resolve, they just belong to that file
        for i in obj.FirstGlobal..obj.ElfSyms.len() {
            let esym = &obj.ElfSyms[i];
            let mut sym = obj.Symbols.get(&i).unwrap().borrow_mut();

            if esym.IsUndef() {
                continue;
            }

            let mut isec = None;
            // absolute symbols dont have related sections
            if !esym.IsAbs() && !esym.IsCommon() {
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

    /// bug?
    pub fn ConvertCommonSymbols(o: &Rc<RefCell<Self>>, ctx: *mut Context) {
        let mut obj = o.borrow_mut();
        if !obj.hasCommon {
            return;
        }
        for i in obj.FirstGlobal..obj.ElfSyms.len() {
            let esym = obj.ElfSyms[i].clone();
            let sym = obj.Symbols.get(&i).unwrap().clone();
            let mut sym = sym.borrow_mut();
            if !esym.IsCommon() {
                continue;
            }

            if let Some(file) = &sym.File {
                let p1 = file.as_ptr() as *const _;
                let p2 = o.as_ptr() as *const _;
                if !std::ptr::eq(p1, p2) {
                    let name = &sym.Name;
                    warn!("{name}: multiple common symbols");
                    continue;
                }

                obj.ElfSections2.push(Default::default());
                let mut shdr = Shdr{..Default::default()};
                let name: String;

                (name, shdr.Flags) = match esym.Type() {
                    abi::STT_TLS =>
                        (".tls_common".into(),
                        (abi::SHF_ALLOC | abi::SHF_WRITE | abi::SHF_TLS) as u64),
                    _ => 
                        (".common".into(),
                        (abi::SHF_ALLOC | abi::SHF_WRITE) as u64)
                };

                shdr.Type = abi::SHT_NOBITS;
                shdr.Size = obj.ElfSyms[i].Size as usize;
                shdr.AddrAlign = obj.ElfSyms[i].Val as usize;

                let idx = obj.ElfSections.len() + obj.ElfSections2.len() - 1;
                drop(esym);
                let isec = InputSection::new(ptr2ref(ctx), name, o.clone(), idx);

                sym.File = Some(o.clone());
                sym.SetInputSection(Some(isec.clone()));
                sym.Value = 0;
                sym.SymIdx = i;
                obj.Sections.push(Some(isec));
            };
        }
        //todo!()
    }

    pub fn MarkLiveObjects(&mut self, roots: &mut Vec<Rc<RefCell<Objectfile>>>) {
        assert!(self.IsAlive);

        for i in self.FirstGlobal..self.ElfSyms.len() {
            let sym = self.Symbols.get(&i).unwrap().borrow();
            let esym = &self.ElfSyms[i];
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

    pub fn ClearSymbols(&mut self) {
        for i in self.FirstGlobal..self.Symbols.len() {
            let sym = self.Symbols.get(&i).unwrap().borrow();
            if let Some(file) = &sym.File {
                if std::ptr::eq(self as *const _, file.as_ptr()) {
//                    warn!("{}: removed.", sym.Name);
                    drop(sym);
                    self.Symbols.remove(&i);
                }
            }
        }
    }

    pub fn RegisterSectionPieces(&mut self) {
        for m in &mut self.MergeableSections {
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

        let len = self.ElfSyms.len();
        for i in 1..len {
            let sym = self.Symbols.get(&i).unwrap();
            let esym = &self.ElfSyms[i];

            if esym.IsAbs() || esym.IsUndef() || esym.IsCommon() {
                continue;
            }

            match &self.MergeableSections[self.GetShndx(esym, i)] {
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
