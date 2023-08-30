//! a further abstractions for elf sections, making it easier to use
use crate::linker::output::GetOutputSection;

use super::elf::ElfGetName;
use super::output::{Chunk, OutputSection};
use super::elf::Shdr;
use super::output::GetOutputName;

use super::common::*;

// an easier-to-use abstraction for Shdr
// the input section is a high level view of abstraction.
// an input section(like .text) consists of some objs and other
// inforamtions, and is described by a Shdr
#[derive(Default,Debug)]
pub struct InputSection {
	pub File: 		Rc<RefCell<Objectfile>>,
	pub	Contents:	Vec<u8>,
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

#[derive(Default,Debug)]
pub struct MergedSection {
	pub Chunk:	Chunk,
	pub Map:	BTreeMap<String, Box<SectionFragment>>,
}

#[derive(Default,Debug, Clone)]
pub struct MergeableSection {
	pub Parent:		Rc<RefCell<MergedSection>>,
	pub P2Align:	u8,
	pub Strs:		Vec<String>,
	pub FragOffset:	Vec<u32>,
	pub Fragments:	Vec<SectionFragment>,
}

#[derive(Default,Debug, Clone)]
pub struct SectionFragment {
	pub OutputSection:	Rc<RefCell<MergedSection>>,
	pub	Offset:			u32,
	pub P2Align:		u8,
	pub	IsAlive:		bool,
}

impl Deref for MergedSection {
	type Target = Chunk;
	fn deref(&self) -> &Self::Target {
		&self.Chunk
	}
}

impl DerefMut for MergedSection {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.Chunk
	}
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
		s.Contents = s.File.borrow().Contents[start..end].into();

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
        let obj = self.File.borrow();
		assert!(self.Shndx < obj.ElfSections.len());
        unsafe {
            std::ptr::addr_of!(obj.ElfSections[self.Shndx]).as_ref().unwrap()
        }
	}

	pub fn Name(&self) -> String {
		ElfGetName(&self.File.borrow().Shstrtab, self.Shdr().Name as usize)
	}

	pub fn WriteTo(&mut self, buf: &mut [u8]) {
		if self.Shdr().Type != abi::SHT_NOBITS && self.ShSize != 0 {
			self.CopyContents(buf);
		}
	}

	// mark
	fn CopyContents(&mut self, buf: &mut [u8]) {
		buf[..self.Contents.len()].copy_from_slice(&self.Contents[..]);
	}
}

impl MergeableSection {
	pub fn new() -> Box<Self> {
		Box::new(MergeableSection{..Default::default()})
	}

	pub fn GetFragment(&self, offset: u32) -> (Option<Box<SectionFragment>>, u32) {
		let pos = self.FragOffset.iter().position(|x| offset < *x ).unwrap_or(self.FragOffset.len());

		if pos == 0 {
			return (None, 0);
		}

		let idx = pos - 1;
		return (
			Some(Box::new(self.Fragments[idx].clone())),
			offset - self.FragOffset[idx]
		);
	}
}

impl MergedSection {
	pub fn new(name: &str, flags: u64, ty: u32) -> Rc<RefCell<MergedSection>> {
		let mut m = MergedSection {
			Chunk: Chunk::new(),
			..Default::default()
		};

		m.Name = name.into();
		m.Shdr.Flags = flags;
		m.Shdr.Type = ty;

		m.ToRcRefcell()
	}

	pub fn GetInstance(ctx: &mut Context, name: &str, ty: u32, flags: u64) -> Rc<RefCell<Self>> {
		let name = GetOutputName(name, flags);
		// ignore these flags
		let flags = flags &
			!abi::SHF_GROUP as u64 & !abi::SHF_MERGE as u64 &
			!abi::SHF_STRINGS as u64 & !abi::SHF_COMPRESSED as u64;

		let osec = ctx.MergedSections.iter().find(
			|osec| {
				let osec = osec.borrow();
				name == osec.Name && flags == osec.Shdr.Flags && ty == osec.Shdr.Type
			}
		);

		match osec {
			Some(o) => o.clone(),
			None => {
				let osec = MergedSection::new(&name, flags, ty);
				ctx.MergedSections.push(osec.clone());
				osec
			}
		}
	}

	pub fn Insert(m: Rc<RefCell<Self>>, key: String, p2align: u8) -> Box<SectionFragment> {
		let mut ms = m.borrow_mut();
		let exist = ms.Map.get(&key).is_some();

		let mut frag;
		if !exist {
			frag = SectionFragment::new(m.clone());
			ms.Map.insert(key, frag.clone());
		}
		else {
			frag = ms.Map.get(&key).unwrap().clone();
		}

		frag.P2Align = frag.P2Align.max(p2align);
		frag.clone()
	}

}

impl SectionFragment {
	pub fn new(m: Rc<RefCell<MergedSection>>) -> Box<Self>{
		Box::new(
			Self {
				OutputSection: m.clone(),
				Offset: u32::MAX,
				..Default::default()
			}
		)
	}
}

/// drop input section's mutable borrow before calling this fn...
pub fn SplitSection(ctx: &mut Context, isec: Rc<RefCell<InputSection>>) -> Box<MergeableSection> {
	let mut m = MergeableSection::new();

	let isec = isec.borrow();
	let shdr = isec.Shdr();
	m.Parent = MergedSection::GetInstance(ctx, &isec.Name(), shdr.Type, shdr.Flags);
	m.P2Align = isec.P2Align;

	let mut data = &isec.Contents[..];
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