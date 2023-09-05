use super::elf::Sym;
use super::inputsections::{InputSection, SectionFragment};
use super::common::*;

// an easier-to-use abstraction for Sym
#[derive(Default,Debug)]
pub struct  Symbol {
	pub File: 				Option<Rc<RefCell<Objectfile>>>,
	pub InputSection:		Option<Rc<RefCell<InputSection>>>,
	pub Name:				String,
	pub Value:				u64,
	pub SymIdx:				usize,
	pub SectionFragment:	Option<Rc<RefCell<SectionFragment>>>,
	pub GotTpIdx:			usize,
	pub Flags:				u32,
}

impl Symbol {
	pub fn new(name: &str) -> Rc<RefCell<Self>> {
			Rc::new(RefCell::new(
			Symbol { Name: name.into(),  ..default()}
		))
	}


	pub fn SetInputSection(&mut self, isec: Option<Rc<RefCell<InputSection>>>) {
		self.InputSection = isec;
		self.SectionFragment = None;
	}

	pub fn SetSectionFragment(&mut self, frag: Option<Rc<RefCell<SectionFragment>>>) {
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
				default()
			}
		}
    }

	pub fn GetAddr(&self) -> u64 {
		if let Some(frag) = &self.SectionFragment {
			return frag.borrow().GetAddr() + self.Value;
		}
		if let Some(isec) = &self.InputSection {
			return unsafe {&*isec.as_ptr()}.GetAddr() + self.Value;
		}
		return self.Value;
	}

	pub fn GetGotTpAddr(&self, ctx: &Context) -> u64 {
		ctx.Got.Shdr.Addr + (self.GotTpIdx as u64) * 8
	}
}