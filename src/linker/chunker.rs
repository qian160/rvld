use super::common::*;
use super::elf::MAGIC;
use super::output::{
    Chunk, MergedSection, OutputEhdr, OutputShdr, OutputSection,
    GetEntryAddr, GetFlags, ptr2ref_dyn
};

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
		ehdr.Entry = GetEntryAddr(ctx);
		ehdr.ShOff = ptr2ref(ctx).Shdr.Shdr.Offset as u64;
		ehdr.Flags = GetFlags(ctx);
		ehdr.EhSize = EHDR_SIZE as u16;
		ehdr.PhEntSize = PHDR_SIZE as u16;

		ehdr.ShEntSize = SHDR_SIZE as u16;
		ehdr.ShNum = (ptr2ref(ctx).Shdr.Shdr.Size / SHDR_SIZE) as u16;

		debug!("\n{:?}", ehdr);

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
	fn CopyBuf(&mut self, ctx: *mut Box<Context>) {
		let ctx = ptr2ref(ctx);
		let base = &mut ctx.Buf[self.Shdr.Offset..];
		Write::<Shdr>(base, &Shdr{..Default::default()});
		// write output file's shdr
		for c in &mut ctx.Chunks {
			let c = ptr2ref_dyn(*c);
			if c.GetShndx() > 0 {
				Write::<Shdr>(
					&mut base[c.GetShndx() * SHDR_SIZE..],
						c.GetShdr()
				);
			}
		}
	}

	// mark
	fn UpdateShdr(&mut self, ctx: *mut Box<Context>) {
		let ctx = ptr2ref(ctx);
		// all 0s at present
		let n = ctx.Chunks.iter()
			.map(|chunk| unsafe {&mut **chunk}.GetShndx())
			.max()
			.unwrap_or(0);

		self.Shdr.Size = (n + 1) * SHDR_SIZE;
	}

	fn GetName(&self) -> &String { &self.Chunk.GetName() }
	fn GetShdr(&mut self) -> &mut Shdr 	{ self.Chunk.GetShdr() }
	fn GetShndx(&self) -> usize { self.Chunk.GetShndx() }
}

impl Chunker for OutputSection {
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
	fn GetName(&self) -> &String { &self.Chunk.GetName() }
	fn GetShdr(&mut self) -> &mut Shdr  { self.Chunk.GetShdr() }
	fn GetShndx(&self) -> usize { self.Chunk.GetShndx() }
}

impl Chunker for MergedSection {
	fn CopyBuf(&mut self, ctx: *mut Box<Context>) {
        let ctx = ptr2ref(ctx);
        let buf = &mut ctx.Buf[self.Shdr.Offset..];
        for (key, frag) in &self.Map {
            let start = frag.Offset as usize;
            buf[start..start + key.len()].copy_from_slice(key.as_bytes());
        }
	}

	fn UpdateShdr(&mut self, _ctx: *mut Box<Context>) {}
	fn GetName(&self) -> &String { self.Chunk.GetName() }
	fn GetShdr(&mut self) -> &mut Shdr { self.Chunk.GetShdr() }
	fn GetShndx(&self) -> usize { self.Chunk.GetShndx() }

}
