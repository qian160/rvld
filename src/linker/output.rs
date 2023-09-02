use elf::abi::EF_RISCV_RVC;
use super::common::*;
use super::inputsections::{InputSection, SectionFragment};

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

#[derive(Debug, Default, Clone)]
pub struct OutputSection {
	pub Chunk:		Chunk,
	pub Members:	Vec<Rc<RefCell<InputSection>>>,
	// shndx? 
	pub Idx:		usize,
}

#[derive(Default,Debug)]
pub struct MergedSection {
	pub Chunk:	Chunk,
	/// note: key is not always strings
	pub Map:	BTreeMap<String, Box<SectionFragment>>,
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
				..Default::default()
			},
            ..Default::default()
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
			..Default::default()
		};

		Chunk.Shdr = shdr;
		Box::new(OutputEhdr { Chunk })
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


impl OutputShdr {
	pub fn new() -> Box<Self> {
		let mut o = OutputShdr{Chunk: Chunk::new()};
		o.Shdr.AddrAlign = 8;
		Box::new(o)
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

	pub fn AssignOffsets(&mut self) {
		//struct fragment {
		//	pub Key: String,
		//	pub val: *const SectionFragment,
		//};
		//let mut fragments: Vec<fragment> = vec![];

		let mut offset = 0;
		let mut p2align = 0;
		for (key, frag) in &mut self.Map {
			offset = AlignTo(offset, 1 << frag.P2Align);
			frag.Offset = offset as u32;
			offset += key.len();
			p2align = p2align.max(frag.P2Align);
		}
		self.Shdr.Size = AlignTo(offset, 1 << p2align);
		self.Shdr.AddrAlign = 1 << p2align;
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
		if Rc::ptr_eq(obj, &ctx.InternalObj) {
			continue;
		}

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
