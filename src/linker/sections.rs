//! a further abstractions for elf sections, making it easier to use
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use super::ElfGetName;
use super::objectfile::Objectfile;
use super::elf::Shdr;

// a easier-to-use abstraction for Shdr
#[derive(Default,Debug)]
pub struct InputSection {
	pub File: 		Rc<RefCell<Objectfile>>,
	pub	Contents:	Vec<u8>,
	pub Shndx:		usize,
}


impl InputSection {
	pub fn new(file: Rc<RefCell<Objectfile>>, shndx: usize) -> Rc<RefCell<Self>>{
		let mut s = InputSection {
			File: file,
			Shndx: shndx,
			Contents: vec![],
		};

		let shdr = s.Shdr();
		let start = shdr.Offset as usize;
		let end = (shdr.Offset + shdr.Size) as usize;
		s.Contents = s.File.borrow().borrow().Contents[start..end].into();

		Rc::new(RefCell::new(s))
	}

	pub fn Shdr(&self) -> &Shdr {
        let binding = self.File.borrow();
        let f = binding.borrow();
		assert!(self.Shndx < f.ElfSections.len());
        unsafe {
            std::ptr::addr_of!(f.ElfSections[self.Shndx]).as_ref().unwrap()
        }
	}

	pub fn Name(&self) -> String {
		ElfGetName(&self.File.borrow().borrow().Shstrtab, self.Shdr().Name as usize)
	}
}