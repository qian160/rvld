use std::cell::RefCell;
use std::rc::Rc;

use crate::error;

use super::context::Context;
use super::elf::Sym;
use super::objectfile::Objectfile;
use super::sections::InputSection;

// a easier-to-use abstraction for Sym
#[derive(Default,Debug)]
pub struct  Symbol {
	pub File: 			Option<Rc<RefCell<Objectfile>>>,
	pub InputSection:	Option<Rc<RefCell<InputSection>>>,
	pub Name:			String,
	pub Value:			u64,
	pub SymIdx:			usize,
}

impl Symbol {
	pub fn new(name: &str) -> Rc<RefCell<Self>> {
			Rc::new(RefCell::new(
			Symbol { Name: name.into(),  ..Default::default()}
		))
	}

	pub fn SetInputSection(&mut self, isec: Option<Rc<RefCell<InputSection>>>) {
		self.InputSection = isec;
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
                f.borrow().borrow().IsAlive
            }
            else {
                false
            }
        }
	}

    pub fn ElfSym(&self) -> Rc<Sym> {
        if let Some(file) = &self.File {
            let o = file.borrow();
            let f = o.borrow();
            assert!(self.SymIdx < f.ElfSyms.len());
            return f.ElfSyms[self.SymIdx].clone();
        }
        error!("1");
        Default::default()
    }

	pub fn Clear(&mut self) {
		*self = Symbol{..Default::default()};
		//self.File = None; 
		//self.InputSection = None;
		//self.SymIdx = usize::MAX;
	}
}