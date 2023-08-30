use super::common::*;
use super::output::{OutputEhdr, OutputShdr, Chunker};
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
    //// the first section header is always empty.(according to the abi)
    ctx.Chunks.push(std::ptr::addr_of_mut!(*ctx.Shdr));
}

pub fn GetFileSize(ctx: &mut Context) -> usize {
    let mut off = 0;
    for c in &mut ctx.Chunks {
        let c = unsafe { &mut **c};
        off = AlignTo(off, c.GetShdr().AddrAlign);
        c.GetShdr().Offset = off;
        off += c.GetShdr().Size;
    }
    off
}

// mark. there's probably a bug here
/// fill up the `Members` field for ctx.outputsections
pub fn BinSections(ctx: &mut Context) {
    let len = ctx.OutputSections.len();
    // this will make all the Rc point to a same addressbugs
    //
    //ctx.OutputSections = vec![Default::default(); len];
    // this works fine but not so elegant
    //for i in 0..len{
    //    ctx.OutputSections.push(Default::default());
    //}
    ctx.OutputSections = (0..len)
        .map(|_| Default::default())
        .collect();

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