use crate::warn;

use super::context::Context;
use super::objectfile::Objectfile;

pub fn ResolveSymbols(ctx: &mut Context) {

    for (i, file) in ctx.Objs.iter().enumerate() {
        //let sz = file.borrow().borrow().Contents.len();
        //for sym in file.borrow().borrow().ElfSyms.iter() {
        //    crate::debug!("\n{:?}", sym);
        //}
        //info!(" #{} - {}, {}", i, file.borrow().Name(), sz);
        Objectfile::ResolveSymbols(file);
    }

    MarkLiveObjects(ctx);

    for file in &ctx.Objs {
        if file.borrow().IsAlive() == false {
            Objectfile::ClearSymbols(file);
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
        Objectfile::MarkLiveObjects(&file, ctx, &mut roots);
        roots = roots[1..].into();
    }
}