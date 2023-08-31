use super::common::*;
use super::elf::IMAGE_BASE;
use super::output::{OutputEhdr, OutputShdr, Chunker, ptr2ref_dyn};
use super::symbol::Symbol;

pub fn ResolveSymbols(ctx: &mut Context) {
    for file in ctx.Objs.iter() {
        Objectfile::ResolveSymbols(file);
    }

    MarkLiveObjects(ctx);

    for file in &ctx.Objs {
        if file.borrow().IsAlive() == false {
            file.borrow_mut().ClearSymbols();
        }
    }
    ctx.Objs.retain(|obj| {obj.borrow().IsAlive()});
}   

pub fn MarkLiveObjects(ctx: &mut Context) {
    let mut roots = vec![];
    for file in &ctx.Objs {
        if file.borrow().IsAlive() {
            roots.push(file.clone());
        }
    }

    assert!(roots.len() > 0);

    while roots.len() > 0 {
        let file = roots[0].clone();
        if file.borrow().IsAlive() == false {
            continue;
        }
        file.borrow_mut().MarkLiveObjects(&mut roots);
        roots = roots[1..].into();
    }
}

pub fn RegisterSectionPieces(ctx: &mut Context) {
    for obj in &ctx.Objs {
        obj.borrow_mut().RegisterSectionPieces();
    }
}

// mark
pub fn CreateInternalFile(ctx: &mut Context) {
    let mut obj = Objectfile{
        ..Default::default()
    };

    obj.Symbols.insert(0, Symbol::new(""));
    obj.FirstGlobal = 1;
    obj.IsAlive = true;
    obj.ElfSyms = ctx.InternalEsyms.clone();

    let o = obj.ToRcRefcell();
    ctx.InternalObj = o.clone();
    // ?
    ctx.Objs.push(o);
}

// mark
pub fn CreateSyntheticSections(ctx: &mut Context) {
    ctx.Ehdr = OutputEhdr::new();
    ctx.Shdr = OutputShdr::new();

    // ehdr must be the first chunk to be written
    ctx.Chunks.push(std::ptr::addr_of_mut!(*ctx.Ehdr));
    // the first section header is always empty.(according to the abi)
    ctx.Chunks.push(std::ptr::addr_of_mut!(*ctx.Shdr));
}

pub fn SetOutputSectionOffsets(ctx: &mut Context) -> usize {
    let mut addr = IMAGE_BASE;
    // set up addr
    for c in &ctx.Chunks {
        let c = ptr2ref_dyn(*c);
        if c.GetShdr().Flags & abi::SHF_ALLOC as u64 == 0 {
            continue;
        }

        addr = AlignTo(addr, c.GetShdr().AddrAlign);
        c.GetShdr().Addr = addr as u64;

        if !isTbss(c) {
            addr += c.GetShdr().Size;
        }
    }

    let mut i = 0;
    let first = ptr2ref_dyn(ctx.Chunks[0]);
    // set up offset
    loop {
        let shdr = ptr2ref_dyn(ctx.Chunks[i]).GetShdr();
        shdr.Offset = (shdr.Addr - first.GetShdr().Addr) as usize;
        i += 1;

        if i >= ctx.Chunks.len() || 
            ptr2ref_dyn(ctx.Chunks[i]).GetShdr().Flags & abi::SHF_ALLOC as u64 == 0 {
            break;
        }
    }

    let lastShdr = ptr2ref_dyn(ctx.Chunks[i-1]).GetShdr();
    let mut fileoff = lastShdr.Offset + lastShdr.Size;

    // non-alloc sections 
    while i < ctx.Chunks.len() {
        let shdr = ptr2ref_dyn(ctx.Chunks[i]).GetShdr();
        fileoff = AlignTo(fileoff, shdr.AddrAlign);
        shdr.Offset = fileoff;
        fileoff += shdr.Size;
        i += 1;
    }
    fileoff
}

// mark. there's probably a bug here
/// fill up the `Members` field for ctx.outputsections
pub fn BinSections(ctx: &mut Context) {
    for file in &ctx.Objs {
        for isec in &file.borrow().Sections {
            match isec {
                None => continue,
                Some(i) => {
                    if i.borrow().IsAlive == false {
                        continue;
                    }

                    let idx = i.borrow().OutputSection.borrow().Idx;
                    ctx.OutputSections[idx].borrow_mut().Members.push(i.clone());
                }
            }
        }
    }
}

//pub fn CollectOutputSections(ctx: &mut Context) -> Vec<Rc<RefCell<dyn Chunker>>>{
pub fn CollectOutputSections(ctx: &mut Context) -> Vec<*mut dyn Chunker>{
    // in fact dyn chunker = outputsections here
    let mut osecs: Vec<*mut dyn Chunker> = vec![];
    for osec in &mut ctx.OutputSections {
        if osec.borrow().Members.len() > 0 {
            let osec_ptr = unsafe {&mut *osec.as_ptr()};
            osecs.push(osec_ptr);
        }
    }
    osecs
}

pub fn ComputeSectionSizes(ctx: &mut Context) {
    for osec in &ctx.OutputSections {
        let mut offset = 0;
        let mut p2align = 0;
        for isec in &osec.borrow().Members {
            offset = AlignTo(offset, 1 << isec.borrow().P2Align);
            isec.borrow_mut().Offset = offset;
            offset += isec.borrow().ShSize;
            p2align = p2align.max(isec.borrow().P2Align);
        }
        //debug!("{offset}");
        osec.borrow_mut().Shdr.Size = offset;
        osec.borrow_mut().Shdr.AddrAlign = 1 << p2align;
    }
}

/// EHDR
/// PHDRs
/// .note
/// alloc sections(.text .rodata .data ...)
/// non-alloc sections (.symtab .debug .strtab ... )
/// SHDRs
pub fn SortOutputSections(ctx: &mut Context) {
    let rank = |c: &mut dyn Chunker| -> u32 {
        let ty = c.GetShdr().Type;
        let flags = c.GetShdr().Flags;
        let eptr = std::ptr::addr_of!(*ctx.Ehdr);
        let sptr = std::ptr::addr_of!(*ctx.Shdr);

        if flags & abi::SHF_ALLOC as u64 == 0 {
            return u32::MAX - 1;
        }
        if std::ptr::eq(std::ptr::addr_of!(*c) as *const OutputShdr, sptr) {
            return u32::MAX;
        }
        if std::ptr::eq(std::ptr::addr_of!(*c) as *const OutputEhdr, eptr) {
            return 0;
        }
        if ty == abi::SHT_NOTE {
            return 2;
        }
        let b2i = |b: bool| -> u32 {
            match b {
                true => 1,
                false => 0
            }
        };

        let writeable = b2i(flags & abi::SHF_WRITE as u64 != 0);
        let notExec = b2i(flags & abi::SHF_EXECINSTR as u64 == 0);
        let notTls = b2i(flags & abi::SHF_TLS as u64 == 0);
        let isBss = b2i(ty == abi::SHT_NOBITS);
        
        return writeable << 7 | notExec << 6 | notTls << 5 | isBss << 4;
    };

    ctx.Chunks.sort_by_key(|c| {
        unsafe {rank(&mut **c)}
    })
}

pub fn isTbss(chunk: &mut dyn Chunker) -> bool {
    let shdr = chunk.GetShdr();
    shdr.Type == abi::SHT_NOBITS && shdr.Flags & abi::SHF_TLS as u64 != 0
}