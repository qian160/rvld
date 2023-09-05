//! a further abstractions for elf sections, making it easier to use

use crate::linker::output::GetOutputSection;

use super::elf::{ElfGetName, Rela};
use super::gotsection::{writeBtype, writeJtype, writeUtype, writeItype, writeStype, setRs1};
use super::output::OutputSection;
use  super::mergedsection::MergedSection;

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
	pub RelsecIdx:	usize,
	pub Rels:		Vec<Rela>
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
			Offset: usize::MAX,
			RelsecIdx: usize::MAX,
			ShSize: usize::MAX,
			..default()
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
		let strtab = unsafe {&mut *self.File.as_ptr()}.Shstrtab.GetSlice();
		ElfGetName(strtab, self.Shdr().Name as usize)
	}

	// bad code. temp
	pub fn GetRels(&mut self) -> Vec<Rela> {
		if self.RelsecIdx == usize::MAX || self.Rels.len() != 0 {
			return self.Rels.clone();
		}
		let f = self.File.borrow();
		let shdr = &f.ElfSections[self.RelsecIdx];
		let bytes = f.GetBytesFromShdr(shdr);
		self.Rels = ReadSlice::<Rela>(bytes);
		self.Rels.clone()
	}

	pub fn GetAddr(&self) -> u64 {
		self.OutputSection.borrow().Shdr.Addr + self.Offset as u64
	}

	pub fn ScanRelocations(&mut self) {
		for rel in self.GetRels() {
			let f = self.File.borrow();
			let sym = f.Symbols.get(&(rel.Sym as usize)).unwrap();
			if sym.borrow().File.is_none() {
				continue;
			}

			if rel.Type == abi::R_RISCV_TLS_GOT_HI20 {
				sym.borrow_mut().Flags |= super::elf::NEEDS_GOT_TP;
			}
		}
	}

	pub fn WriteTo(&mut self, ctx: *mut Box<Context>, buf: &mut [u8]) {
		//let ctx = ptr2ref(ctx);
		if self.Shdr().Type != abi::SHT_NOBITS && self.ShSize != 0 {
			self.CopyContents(buf);
			if self.Shdr().Flags & abi::SHF_ALLOC as u64 != 0 {
				self.ApplyRelocAlloc(ctx, buf);
			}
		}
	}

	// mark
	fn CopyContents(&mut self, buf: &mut [u8]) {
		let slice = self.Contents.GetSlice();
		buf[..self.Contents.1].copy_from_slice(slice);
	}

	fn ApplyRelocAlloc(&mut self, ctx: *mut Box<Context>, base: &mut [u8]) {
		let rels = self.GetRels();
		for rel in &rels {
			if matches!(rel.Type, abi::R_RISCV_NONE | abi::R_RISCV_RELAX) {
				continue;
			}

			let f = self.File.borrow();
			let sym = f.Symbols.get(&(rel.Sym as usize)).unwrap().borrow();
			let loc = &mut base[rel.Offset as usize ..];

			if sym.File.is_none() {
				continue;
			}

			let S = CheckedU64::from(sym.GetAddr());
			let A = CheckedU64::from(rel.Addend);
			let P = CheckedU64::from(self.GetAddr() + rel.Offset);

			match rel.Type {
				abi::R_RISCV_32 => Write(loc, (S+A).0 as u32),
				abi::R_RISCV_64 => Write(loc, S+A),
				abi::R_RISCV_BRANCH => writeBtype(loc,(S+A-P).0 as u32),
				abi::R_RISCV_JAL => writeJtype(loc, (S+A-P).0 as u32),
				abi::R_RISCV_CALL | abi::R_RISCV_CALL_PLT => {
					let val = (S+A-P).0;
					writeUtype(loc, val as u32);
					writeItype(&mut loc[4..], val as u32);
				},
				abi::R_RISCV_TLS_GOT_HI20 => Write(loc, (CheckedU64(sym.GetGotTpAddr(unsafe{&**ctx})) +A-P).0 as u32),
				abi::R_RISCV_PCREL_HI20 => Write(loc, (S+A-P).0 as u32),
				abi::R_RISCV_HI20 => Write(loc, (S+A).0 as u32),
				abi::R_RISCV_LO12_I | abi::R_RISCV_LO12_S => {
					let val = S+A;
					if rel.Type == abi::R_RISCV_LO12_I {
						writeItype(loc, val.0 as u32);
					}
					else {
						writeStype(loc, val.0 as u32)
					}

					if SignExtend(val.0, 11) == val.0 {
						setRs1(loc, 0);
					}
				},
				abi::R_RISCV_TPREL_LO12_I | abi::R_RISCV_TPREL_LO12_S => {
					let val = S + A - CheckedU64(unsafe{&*ctx}.TpAddr);
					if rel.Type == abi::R_RISCV_TPREL_LO12_I {
						writeItype(loc, val.0 as u32);
					}
					else {
						writeStype(loc, val.0 as u32);
					}

					if SignExtend(val.0, 11) == val.0 {
						setRs1(loc, 4);
					}
				},
				_ => {}
			}
		}

		for rel in &rels {
			match rel.Type {
				abi::R_RISCV_PCREL_LO12_I | abi::R_RISCV_PCREL_LO12_S => {
					let f = self.File.borrow();
					let sym = f.Symbols.get(&(rel.Sym as usize)).unwrap().borrow();
					assert!(sym.InputSection.is_some());
					if let Some(isec) = &sym.InputSection {
						assert!(std::ptr::eq(self, isec.as_ptr()));
					}

					let val = Read::<u32>(&base[sym.Value as usize..]);
					let loc = &mut base[rel.Offset as usize..];
					if rel.Type == abi::R_RISCV_PCREL_LO12_I {
						writeItype(loc, val);
					}
					else {
						writeStype(loc, val);
					}
				},
				_ => {}
			}
		}

		for rel in rels {
			match rel.Type {
				abi::R_RISCV_PCREL_HI20 | abi::R_RISCV_TLS_GOT_HI20 => {
					let loc = &mut base[rel.Offset as usize..];
					let val = Read::<u32>(loc);
					Write(loc, val);
					writeUtype(loc, val);
				},
				_ => {}
			}
		}
		//todo!();
	}
}

impl MergeableSection {
	pub fn new() -> Box<Self> {
		Box::new(MergeableSection{..default()})
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
			..default()
		}.ToRcRefcell()
	}
	pub fn GetAddr(&self) -> u64 {
		self.OutputSection.borrow().Shdr.Addr + self.Offset as u64
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