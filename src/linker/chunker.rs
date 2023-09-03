use super::common::*;
use super::elf::MAGIC;
use super::output::{
    Chunk, MergedSection, OutputEhdr, OutputShdr, OutputSection, OutputPhdr,
    GetEntryAddr, GetFlags, ptr2ref_dyn, createPhdr,
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

impl dyn Chunker {
    pub fn toPhdrFlags(&mut self) -> u32 {
        let mut ret = abi::PF_R;
        let write = self.GetShdr().Flags & abi::SHF_WRITE as u64 != 0;
        let exec = self.GetShdr().Flags & abi::SHF_EXECINSTR as u64 != 0;
        if write {
            ret |= abi::PF_W;
        }
        if exec {
            ret |= abi::PF_X;
        }

        ret
    }
    pub fn isTls(&mut self) -> bool {
        self.GetShdr().Flags & abi::SHF_TLS as u64 != 0
    }
    pub fn isBss(&mut self) -> bool {
        self.GetShdr().Type == abi::SHT_NOBITS && !self.isTls()
    }
    pub fn isNote(&mut self) -> bool {
        let shdr = self.GetShdr();
        shdr.Type == abi::SHT_NOTE && shdr.Flags & abi::SHF_ALLOC as u64 != 0
    }
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
        ehdr.PhOff = ptr2ref(ctx).Phdr.Shdr.Offset as u64;
		ehdr.ShOff = ptr2ref(ctx).Shdr.Shdr.Offset as u64;
		ehdr.Flags = GetFlags(ctx);
		ehdr.EhSize = EHDR_SIZE as u16;
		ehdr.PhEntSize = PHDR_SIZE as u16;

        ehdr.PhNum = (ptr2ref(ctx).Phdr.Shdr.Size / PHDR_SIZE) as u16;
		ehdr.ShEntSize = SHDR_SIZE as u16;
		ehdr.ShNum = (ptr2ref(ctx).Shdr.Shdr.Size / SHDR_SIZE) as u16;

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

impl Chunker for OutputPhdr {
    fn CopyBuf(&mut self, ctx: *mut Box<Context>) {
        // bug here. write fails to work?
        let buf = &mut ptr2ref(ctx).Buf[self.Shdr.Offset..];
        let data = vec2slice(&self.Phdrs);
        buf[..self.Shdr.Size].copy_from_slice(data);
    }

    fn UpdateShdr(&mut self, ctx: *mut Box<Context>) {
        let c = Box::as_mut(unsafe{&mut *ctx});
        self.Phdrs = createPhdr(c);
        self.Shdr.Size = self.Phdrs.len() * PHDR_SIZE;
    }

    fn GetName(&self) -> &String { self.Chunk.GetName() }
    fn GetShdr(&mut self) -> &mut Shdr { self.Chunk.GetShdr() }
    fn GetShndx(&self) -> usize { self.Chunk.GetShndx() }
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
            let start = frag.borrow().Offset as usize;
            buf[start..start + key.len()].copy_from_slice(key.as_bytes());
        }
	}

	fn UpdateShdr(&mut self, _ctx: *mut Box<Context>) {}
	fn GetName(&self) -> &String { self.Chunk.GetName() }
	fn GetShdr(&mut self) -> &mut Shdr { self.Chunk.GetShdr() }
	fn GetShndx(&self) -> usize { self.Chunk.GetShndx() }

}
