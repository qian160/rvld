use crate::utils::AlignTo;
#[allow(unused)]
use crate::warn;

use super::common::*;
use super::output::OutputEhdr;
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

pub fn CreateInternalFile(ctx: &mut Context) {
    let mut obj = Objectfile{
        ..Default::default()
    };

    //ctx.Objs.push(value)
    obj.Symbols.insert(0, Symbol::new(""));
    obj.FirstGlobal = 1;
    obj.IsAlive = true;
    obj.ElfSyms = ctx.InternalEsyms.clone();
    ctx.InternalObj = Box::new(obj);
}

pub fn CreateSections(ctx: &mut Context) {
    ctx.Ehdr = OutputEhdr::new();
    ctx.Chunks.push(ctx.Ehdr.clone());
}

pub fn GetFileSize(ctx: &Context) -> usize {
    let mut off = 0;
    for c in &ctx.Chunks {
        off = AlignTo(off, c.GetShdr().AddrAlign);
        off += c.GetShdr().Size;
    }

    off
}