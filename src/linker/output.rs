use super::elf::{Shdr, EHDR_SIZE, Ehdr, MAGIC, PHDR_SIZE, SHDR_SIZE};
use super::common::*;

#[derive(Default,Debug, Clone)]
pub struct Chunk {
    pub Name:   String,
    pub Shdr:   Shdr,
}

pub trait Chunker {
	fn GetShdr(&self) -> &Shdr;
	fn CopyBuf(&mut self, buffer: &mut Vec<u8>);
}

impl Chunker for Chunk {
	fn CopyBuf(&mut self, _buffer: &mut Vec<u8>) { /*parent defined*/ }
	fn GetShdr(&self) -> &Shdr {
		&self.Shdr
	}
}

#[derive(Default, Clone)]
pub struct OutputEhdr {
	pub Chunk: Chunk
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            Shdr: Shdr{
				AddrAlign: 1,
				..Default::default()
			},
            Name: "".into()
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

impl Chunker for OutputEhdr {
	// need first to allocate some spaces for buffer...
	fn CopyBuf(&mut self, buffer: &mut Vec<u8>){
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
		buffer.copy_from_slice(unsafe{
			std::slice::from_raw_parts(ehdr_ptr, EHDR_SIZE)
		});
	}
	fn GetShdr(&self) -> &Shdr {
		self.Chunk.GetShdr()
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