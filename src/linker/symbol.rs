use super::elf::Sym;
use super::sections::{InputSection, SectionFragment};

use super::common::*;

// an easier-to-use abstraction for Sym
#[derive(Default,Debug)]
pub struct  Symbol {
	pub File: 				Option<Rc<RefCell<Objectfile>>>,
	pub InputSection:		Option<Rc<RefCell<InputSection>>>,
	pub Name:				String,
	pub Value:				u64,
	pub SymIdx:				usize,
	pub SectionFragment:	Option<Box<SectionFragment>>,
}

impl Symbol {
	pub fn new(name: &str) -> Rc<RefCell<Self>> {
			Rc::new(RefCell::new(
			Symbol { Name: name.into(),  ..Default::default()}
		))
	}


	pub fn SetInputSection(&mut self, isec: Option<Rc<RefCell<InputSection>>>) {
		self.InputSection = isec;
		self.SectionFragment = None;
	}

	pub fn SetSectionFragment(&mut self, frag: Option<Box<SectionFragment>>) {
		self.InputSection = None;
		self.SectionFragment = frag;
	}


	pub fn GetSymbolByName(ctx: &mut Context, name: &str) -> Rc<RefCell<Symbol>> {
		if let Some(sym) = ctx.SymbolMap.get(name.into()) {
			return sym.clone();
		}
		let newSym = Symbol::new(name.into());
		ctx.SymbolMap.insert(name.into(), newSym.clone());
		return newSym.clone();
	}

	pub fn FileAlive(&self) -> bool {
		// external global symbols?
        if self.File.is_none() {
            false
        }
        else {
            if let Some(f) = &self.File {
                f.borrow().IsAlive
            }
            else {
                false
            }
        }
	}

    pub fn ElfSym(&self) -> Rc<Sym> {
		match &self.File {
			Some(file) => {
				let o = file.borrow();
				assert!(self.SymIdx < o.ElfSyms.len());
				o.ElfSyms[self.SymIdx].clone()
			},
			None => {
				error!("should not happen...");
				Default::default()
			}
		}
    }
}