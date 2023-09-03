//! a further abstractions for elf sections, making it easier to use

use crate::linker::output::GetOutputSection;

use super::elf::ElfGetName;
use super::output::{OutputSection, MergedSection};

use super::common::*;

// an easier-to-use abstraction for Shdr
// the input section is a high level view of abstraction.
// an input section(like .text) consists of some objs and other
// inforamtions, and is described by a Shdr
#[derive(Default, Debug)]
pub struct InputSection {
	pub File: 		Rc<RefCell<Objectfile>>,
	// no copy, but still works slowly?
	pub Contents:	ByteSequence,
	//pub	Contents:	Vec<u8>,
	pub Shndx:		usize,
	/// the `Size` field from shdr
	pub ShSize:  	usize,
	pub IsAlive:  	bool,
	pub P2Align:  	u8,
	/// used by output section
	pub Offset:		usize,
	/// multiple inputsecs could be mapped to the same outputsec
	pub OutputSection:	Rc<RefCell<OutputSection>>,
}

#[derive(Default,Debug, Clone)]
pub struct MergeableSection {
	pub Parent:		Rc<RefCell<MergedSection>>,
	pub P2Align:	u8,
	pub Strs:		Vec<String>,
	pub FragOffset:	Vec<u32>,
	pub Fragments:	Vec<Rc<RefCell<SectionFragment>>>,
}

#[derive(Default,Debug, Clone)]
pub struct SectionFragment {
	pub OutputSection:	Rc<RefCell<MergedSection>>,
	pub	Offset:			u32,
	pub P2Align:		u8,
	pub	IsAlive:		bool,
}

impl InputSection {
	pub fn new(ctx: &mut Context, name: String, file: Rc<RefCell<Objectfile>>, shndx: usize) -> Rc<RefCell<Self>>{
		let mut s = InputSection {
			File: file,
			Shndx: shndx,
			IsAlive: true,
			..Default::default()
		};

		let shdr = s.Shdr().clone();
		assert!(shdr.Flags & abi::SHF_COMPRESSED as u64 == 0);

		let start = shdr.Offset;
		let end = shdr.Offset + shdr.Size;
		// to avoid borrow checks...
		let ptr = unsafe {&*s.File.as_ptr()};
		let start_ptr = ptr.Contents[start..].as_ptr();
		s.Contents = ByteSequence::new(start_ptr, end - start);

		s.ShSize = shdr.Size;
		s.P2Align = match shdr.AddrAlign {
			0 => 0,
			_ => shdr.AddrAlign.trailing_zeros() as u8
		};
		s.OutputSection = GetOutputSection(ctx, name, shdr.Type, shdr.Flags);
		s.ToRcRefcell()
	}

	/// this function is not so friendly...we must not own any
	/// mutable borrow of that objectfile before calling this fn
	pub fn Shdr(&self) -> &Shdr {
		let obj = unsafe { &*self.File.as_ptr() };
		if self.Shndx < obj.ElfSections.len() {
			return &obj.ElfSections[self.Shndx];
		}
		return &obj.ElfSections2[self.Shndx - obj.ElfSections.len()];
	}

	pub fn Name(&self) -> String {
		ElfGetName(&self.File.borrow().Shstrtab.GetSlice(), self.Shdr().Name as usize)
	}

	pub fn WriteTo(&mut self, buf: &mut [u8]) {
		if self.Shdr().Type != abi::SHT_NOBITS && self.ShSize != 0 {
			self.CopyContents(buf);
		}
	}

	// mark
	fn CopyContents(&mut self, buf: &mut [u8]) {
		let slice = self.Contents.GetSlice();
		buf[..self.Contents.1].copy_from_slice(slice);
	}
}

impl MergeableSection {
	pub fn new() -> Box<Self> {
		Box::new(MergeableSection{..Default::default()})
	}

	pub fn GetFragment(&self, offset: u32) -> (Option<Rc<RefCell<SectionFragment>>>, u32) {
		let pos = self.FragOffset.iter().position(|x| offset < *x ).unwrap_or(self.FragOffset.len());

		if pos == 0 {
			return (None, 0);
		}

		let idx = pos - 1;
		return (
			Some(self.Fragments[idx].clone()),
			offset - self.FragOffset[idx]
		);
	}
}

impl SectionFragment {
	pub fn new(m: Rc<RefCell<MergedSection>>) -> Rc<RefCell<Self>>{
		Self {
			OutputSection: m.clone(),
			Offset: u32::MAX,
			..Default::default()
		}.ToRcRefcell()
	}
}

/// drop input section's mutable borrow before calling this fn...
pub fn SplitSection(ctx: &mut Context, isec: Rc<RefCell<InputSection>>) -> Box<MergeableSection> {
	let mut m = MergeableSection::new();

	let isec = isec.borrow();
	let shdr = isec.Shdr();
	m.Parent = MergedSection::GetInstance(ctx, &isec.Name(), shdr.Type, shdr.Flags);
	m.P2Align = isec.P2Align;

	let mut data = isec.Contents.GetSlice();
	let mut offset = 0;
	if shdr.Flags & abi::SHF_STRINGS as u64 != 0 {
		while data.len() > 0 {
			let end = FindNull(&data, shdr.EntSize);

			let sz = end + shdr.EntSize;
			let substr = unsafe {std::str::from_utf8_unchecked(&data[..sz]).into()};
//			debug!("\"{substr}\"");
			data = &data[sz..];
			m.Strs.push(substr);
			m.FragOffset.push(offset);
			offset += sz as u32;
		}
	}
	else {
		if data.len() % shdr.EntSize as usize != 0 {
			error!("section size is not multiple of entsize");
		}
		while data.len() > 0 {
			let subdata: String = unsafe {std::str::from_utf8_unchecked(&data[..shdr.EntSize]).into()};
			data = &data[shdr.EntSize..];
			// not string in fact
			m.Strs.push(subdata);
			m.FragOffset.push(offset);
			offset += shdr.EntSize as u32;
		}
	}
	m
}


/// used by SplitSection
/// entsize means some kind of alignment...
/// entsize = 1 => "foo\0"
/// entsize = 4 => "f\0\0\0o\0\0\0o\0\0\0\0\0\0\0"
pub fn FindNull(data: &[u8], entSize: usize) -> usize {
    if entSize == 1 {
        return data.iter().position(|x| *x == 0 ).unwrap();
    }

    for i in (0..data.len()).step_by(entSize) {
        let bytes = &data[i..i + entSize];
        if bytes.iter().all(|x| *x == 0) {
            return i;
        }
    }
    error!("string is not null terminated!");
    1145141919810
}