use elf::abi::EF_RISCV_RVC;
use crate::linker::elf::PAGESIZE;

use super::common::*;
use super::inputsections::InputSection;

/// an abstract base writting unit
#[derive(Default,Debug, Clone)]
pub struct Chunk {
    pub Name:   String,
	/// this is only used during linking, and will not be put into target file
    pub Shdr:   Shdr,
	pub Shndx:	usize,
}

#[derive(Default, Clone)]
pub struct OutputEhdr {
	pub Chunk: Chunk
}

#[derive(Default, Clone)]
pub struct OutputShdr {
	pub Chunk: Chunk
}


#[derive(Default, Clone)]
pub struct OutputPhdr {
	pub Chunk: 	Chunk,
	pub Phdrs:	Vec<Phdr>,
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

impl Deref for OutputPhdr {
	type Target = Chunk;
	fn deref(&self) -> &Self::Target {
		&self.Chunk
	}
}

impl DerefMut for OutputPhdr {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.Chunk
	}
}

impl Deref for OutputEhdr {
	type Target = Chunk;
	fn deref(&self) -> &Self::Target {
		&self.Chunk
	}
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            Shdr: Shdr{
				AddrAlign: 1,
				..default()
			},
            ..default()
        }
    }
}

impl OutputEhdr {
	pub fn new() -> Box<Self> {
		let mut Chunk = Chunk::new();
		let shdr = Shdr{
			Flags:		abi::SHF_ALLOC as u64,
			Size:		EHDR_SIZE,
			AddrAlign:	8,
			..default()
		};

		Chunk.Shdr = shdr;
		Box::new(OutputEhdr { Chunk })
	}
}

impl OutputSection {
	pub fn new(name: String, ty: u32, flags: u64, idx: usize) -> Box<Self> {
		let mut o = Self {
			Chunk: Chunk::new(),
			..default()
		};
		o.Name = name;
		o.Shdr.Type = ty;
		o.Shdr.Flags = flags;
		o.Idx = idx;

		Box::new(o)
	}
}


impl OutputShdr {
	pub fn new() -> Box<Self> {
		let mut o = OutputShdr{Chunk: Chunk::new()};
		o.Shdr.AddrAlign = 8;
		Box::new(o)
	}
}

impl OutputPhdr {
	pub fn new() -> Box<Self> {
		let mut o = Self {
			Chunk: Chunk::new(),
			..default()
		};
		o.Shdr.Flags = abi::SHF_ALLOC as u64;
		o.Shdr.AddrAlign = 8;

		Box::new(o)
	}
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

pub fn GetEntryAddr(ctx: *mut Box<Context>) -> u64 {
	let ctx = ptr2ref(ctx);
	for osec in &ctx.OutputSections {
		if osec.borrow().Name == ".text" {
			return osec.borrow().Shdr.Addr;
		}
	}
	0
}

pub fn GetFlags(ctx: *mut Box<Context>) -> u32 {
	let ctx = ptr2ref(ctx);
	assert!(ctx.Objs.len() > 0);
	let mut flags = ctx.Objs[0].borrow().GetEhdr().Flags;
	for obj in &ctx.Objs {
		if obj.borrow().GetEhdr().Flags & EF_RISCV_RVC != 0 {
			flags |= EF_RISCV_RVC;
			break;
		}
	}

	flags
}

pub fn ptr2ref_dyn(ptr: *mut dyn Chunker) -> &'static mut dyn Chunker {
	unsafe {&mut *ptr}
}

pub fn createPhdr(ctx: &mut Context) -> Vec<Phdr> {
	let vec = RefCell::new(vec![]);

	let define = |Type: u32, Flags: u32, minAlign: u64, chunk: &mut dyn Chunker| {
		let shdr = chunk.GetShdr();
		let mut phdr = Phdr{
			Type, Flags,
			Align: minAlign.max(shdr.AddrAlign),
			Offset:  shdr.Offset as u64,
			VAddr:   shdr.Addr,
			PAddr:   shdr.Addr,
			MemSize: shdr.Size as u64,
			..default()
		};

		phdr.FileSize = match shdr.Type {
			abi::SHT_NOBITS => 0,
			_ => shdr.Size as u64,
		};
		vec.borrow_mut().push(phdr);
	};

	let push = |chunk: &mut dyn Chunker| {
		let shdr = chunk.GetShdr();
		let len = vec.borrow().len();
		let mut phdr = &mut vec.borrow_mut()[len - 1];
		phdr.Align = phdr.Align.max(shdr.AddrAlign);

		if shdr.Type != abi::SHT_NOBITS {
			phdr.FileSize = shdr.Addr + shdr.Size as u64 - phdr.VAddr;
		}

		phdr.MemSize = shdr.Addr + shdr.Size as u64 - phdr.VAddr;
	};

	// the 1st phdr should point to the phdr table itself
	define(abi::PT_PHDR, abi::PF_R, 8, &mut *ctx.Phdr);

	let end = ctx.Chunks.len();
	let mut i = 0;
	while i < end {
		let first = ptr2ref_dyn(ctx.Chunks[i]);
		i += 1;
		if !first.isNote() {
			continue;
		}

		let flags = first.toPhdrFlags();
		let alignment = first.GetShdr().AddrAlign;
		define(abi::PT_NOTE, flags, alignment, first);
		while i < end {
			let chunk = ptr2ref_dyn(ctx.Chunks[i]);
			if !chunk.isNote() || !chunk.toPhdrFlags() == flags {
				break;
			}

			push(chunk);
			i += 1;
		}
	}

	// bss
	{
		let mut chunks = ctx.Chunks.clone();
		chunks.retain(|c| {
			!ptr2ref_dyn(*c).isBss()
		});

		let end = chunks.len();
		let mut i = 0;
		while i < end {
			let first = ptr2ref_dyn(chunks[i]);
			i += 1;

			if first.GetShdr().Flags & abi::SHF_ALLOC as u64 == 0 {
				break;
			}

			let flags = first.toPhdrFlags();
			define(abi::PT_LOAD, flags, PAGESIZE, first);

			if !first.isBss() {
				while i < end {
					let c = ptr2ref_dyn(chunks[i]);
					if !(c.toPhdrFlags() == flags) || c.isBss() {
						break;
					}
					push(c);
					i += 1;
				}
			}

			while i < end {
				let c = ptr2ref_dyn(chunks[i]);
				if !c.isBss() || !(c.toPhdrFlags() == flags) {
					break;
				}
				push(c);
				i += 1;
			}
		}
	}

	let mut i = 0;
	while i < ctx.Chunks.len() {
		let c = ptr2ref_dyn(ctx.Chunks[i]);
		if !c.isTls() {
			i += 1;
			continue;
		}

		define(abi::PT_TLS, c.toPhdrFlags(), 1, c);
		i += 1;

		while i < ctx.Chunks.len() {
			let c = ptr2ref_dyn(ctx.Chunks[i]);
			if !c.isTls() {
				break;
			}

			push(c);
			i += 1;
		}

		let len = vec.borrow().len();
		let phdr = &vec.borrow()[len-1];
		ctx.TpAddr = phdr.VAddr;
		i += 1;
	}

	vec.into_inner()
}
