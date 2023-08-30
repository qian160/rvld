use crate::utils;

use super::elf::{Shdr, EHDR_SIZE, Ehdr, MAGIC, PHDR_SIZE, SHDR_SIZE};
use super::common::*;
use super::sections::InputSection;

/// an abstract base writting unit
#[derive(Default,Debug, Clone)]
pub struct Chunk {
    pub Name:   String,
    pub Shdr:   Shdr,
	pub Shndx:	usize,
}

pub trait Chunker {
	fn GetShdr(&mut self) -> &mut Shdr;
	fn GetName(&self) -> &String;
	fn GetShndx(&self) -> usize;

	// use raw pointer to avoid some borrow checks
	/// get some data from the chunk and copy it to a buffer(usually ctx.Buf)
	fn CopyBuf(&mut self, ctx: *mut Box<Context>);
	/// for output section's shdr
	fn UpdateShdr(&mut self, ctx: *mut Box<Context>);
}

impl Chunker for Chunk {
	fn CopyBuf(&mut self, _ctx: *mut Box<Context>) { /*parent defined*/ }
	fn UpdateShdr(&mut self, _ctx: *mut Box<Context>) {}

	fn GetName(&self) -> &String { &self.Name }
	fn GetShdr(&mut self) -> &mut Shdr 	{ &mut self.Shdr }
	fn GetShndx(&self) -> usize { self.Shndx }
}

/// this will be used for writing output file's ehdr
#[derive(Default, Clone)]
pub struct OutputEhdr {
	pub Chunk: Chunk
}

/// this will be used for writing output file's shdr
#[derive(Default, Clone)]
pub struct OutputShdr {
	pub Chunk: Chunk
}

#[derive(Debug, Default, Clone)]
pub struct OutputSection {
	pub Chunk:		Chunk,
	pub Members:	Vec<Rc<RefCell<InputSection>>>,
	// shndx? 
	pub Idx:		usize,
}

impl Deref for OutputSection {
	type Target = Chunk;
	fn deref(&self) -> &Self::Target {
		&self.Chunk
	}
}

impl DerefMut for OutputSection {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.Chunk
	}
}

impl Deref for OutputShdr{
	type Target = Chunk;
	fn deref(&self) -> &Self::Target {
		&self.Chunk
	}
}

impl DerefMut for OutputShdr {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.Chunk
	}
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            Shdr: Shdr{
				AddrAlign: 1,
				..Default::default()
			},
            ..Default::default()
        }
    }
}

impl Deref for OutputEhdr {
	type Target = Chunk;
	fn deref(&self) -> &Self::Target {
		&self.Chunk
	}
}

impl OutputEhdr {
	pub fn new() -> Box<Self> {
		let mut Chunk = Chunk::new();
		let shdr = Shdr{
			Flags:		abi::SHF_ALLOC as u64,
			Size:		EHDR_SIZE,
			AddrAlign:	8,
			..Default::default()
		};

		Chunk.Shdr = shdr;
		
		Box::new(OutputEhdr { Chunk })
	}
}

impl OutputShdr {
	pub fn new() -> Box<Self> {
		let mut o = OutputShdr{Chunk: Chunk::new()};
		o.Shdr.AddrAlign = 8;
		Box::new(o)
	}
}

impl Chunker for OutputEhdr {
	fn CopyBuf(&mut self, ctx: *mut Box<Context>){
		let mut ehdr = Ehdr{..Default::default()};
		ehdr.Ident[0..4].copy_from_slice(MAGIC);
		ehdr.Ident[abi::EI_CLASS] = abi::ELFCLASS64;
		ehdr.Ident[abi::EI_DATA] = abi::ELFDATA2LSB;
		ehdr.Ident[abi::EI_VERSION] = abi::EV_CURRENT;
		ehdr.Ident[abi::EI_OSABI] = 0;
		ehdr.Ident[abi::EI_ABIVERSION] = 0;
		ehdr.Type = abi::ET_EXEC;
		ehdr.Machine = abi::EM_RISCV;
		ehdr.Version = abi::EV_CURRENT as u32;

		ehdr.EhSize = EHDR_SIZE as u16;
		ehdr.PhEntSize = PHDR_SIZE as u16;

		ehdr.ShEntSize = SHDR_SIZE as u16;

		let ehdr_ptr = std::ptr::addr_of!(ehdr) as *const u8;
		let ctx = ptr2ref(ctx);
		ctx.Buf[0..EHDR_SIZE]
			.copy_from_slice(unsafe{
			std::slice::from_raw_parts(ehdr_ptr, EHDR_SIZE)
		});
	}

	fn UpdateShdr(&mut self, _: *mut Box<Context>) {/* do nothing */}
	fn GetShndx(&self) -> usize { self.Chunk.GetShndx()}
	fn GetShdr(&mut self) -> &mut Shdr 	{ self.Chunk.GetShdr() }
	fn GetName(&self) -> &String { self.Chunk.GetName() }
}

impl Chunker for OutputShdr {
	fn GetName(&self) -> &String { &self.Chunk.GetName() }
	fn GetShdr(&mut self) -> &mut Shdr 	{ self.Chunk.GetShdr() }
	fn GetShndx(&self) -> usize { self.Chunk.GetShndx() }

	fn CopyBuf(&mut self, ctx: *mut Box<Context>) {
		let ctx = ptr2ref(ctx);
		let base = &mut ctx.Buf[self.Shdr.Offset..];
		utils::Write::<Shdr>(base, Shdr{..Default::default()});
		// write output file's shdr
		for c in &mut ctx.Chunks {
			let c = unsafe {&mut **c};
			if c.GetShndx() > 0 {
				utils::Write::<Shdr>(
					&mut base[c.GetShndx() * SHDR_SIZE..],
					c.GetShdr().clone()
				);
			}
		}
	}

	// mark
	fn UpdateShdr(&mut self, ctx: *mut Box<Context>) {
		let ctx = ptr2ref(ctx);
		// all 0s at present
		let n = ctx.Chunks.iter()
			.map(|chunk| unsafe {&mut **chunk}.GetShndx()
			)
			.max()
			.unwrap_or(0);

		warn!("{n}");

		self.Shdr.Size = (n + 1) * SHDR_SIZE;
	}
}

impl OutputSection {
	pub fn new(name: String, ty: u32, flags: u64, idx: usize) -> Box<Self> {
		let mut o = Self {
			Chunk: Chunk::new(),
			..Default::default()
		};
		o.Name = name;
		o.Shdr.Type = ty;
		o.Shdr.Flags = flags;
		o.Idx = idx;

		Box::new(o)
	}
}

impl Chunker for OutputSection {
	fn GetName(&self) -> &String { &self.Chunk.GetName() }
	fn GetShdr(&mut self) -> &mut Shdr  { self.Chunk.GetShdr() }
	//fn GetShndx(&self) -> usize { self.Idx }
	fn GetShndx(&self) -> usize { self.Chunk.GetShndx() }

	fn CopyBuf(&mut self, ctx: *mut Box<Context>) {
		if self.Shdr.Type == abi::SHT_NOBITS {
			return;
		}

		let ctx = ptr2ref(ctx);
		let base = &mut ctx.Buf[self.Shdr.Offset..];
		for isec in &self.Members {
			let mut isec = isec.borrow_mut();
			let buf = &mut base[isec.Offset..];
			isec.WriteTo(buf);
		}
	}

	fn UpdateShdr(&mut self, _ctx: *mut Box<Context>) {}
}

pub const PREFIXES: [&str; 13] = [
	".text.", ".data.rel.ro.", ".data.", ".rodata.", ".bss.rel.ro.", ".bss.",
	".init_array.", ".fini_array.", ".tbss.", ".tdata.", ".gcc_except_table.",
	".ctors.", ".dtors.",
];

pub fn GetOutputName(name: &str, flags: u64) -> String {
	if (name == ".rodata" || name.starts_with(".rodata.")) &&
	flags & abi::SHF_MERGE as u64 != 0 {
		return if flags & abi::SHF_STRINGS as u64!= 0 {
			".rodata.str".into()
		}
		else {
			".rodata.cst".into()	// const
		};
	}

	for prefix in PREFIXES {
		let stem = &prefix[..prefix.len() - 1];	// remove the last '.'
		if name == stem || name.starts_with(prefix) {
			return stem.into();
		}
	}

	name.into()
}

/// return existed os, or create a new one
pub fn GetOutputSection(ctx: &mut Context, name: String, ty: u32, flags: u64) -> Rc<RefCell<OutputSection>> {
	let name = GetOutputName(&name, flags);
	let flags = flags & !abi::SHF_GROUP as u64  
		& !abi::SHF_COMPRESSED as u64 & !abi::SHF_LINK_ORDER as u64;
	
	let res = ctx.OutputSections.iter().find(
		|osec| {
			let osec = osec.borrow();
			name == osec.Name && ty == osec.Shdr.Type && flags == osec.Shdr.Flags
		}
	);

	return match res {
		Some(osec) => osec.clone(),
		None => {
			let idx = ctx.OutputSections.len();
			let osec = (*OutputSection::new(name, ty, flags, idx)).ToRcRefcell();
			ctx.OutputSections.push(osec.clone());
			osec
		}
	}
}

/// an ugly function to deal with rust's borrow rules...
pub fn ptr2ref(ctx_ptr: *mut Box<Context>) -> &'static mut Box<Context> {
	unsafe{&mut *ctx_ptr}
}