//! useful informations collected and will be used during linking
use super::common::*;
use super::elf::MachineType;
use super::gotsection::GotSection;
use super::symbol::Symbol;
use super::output::{OutputEhdr, OutputShdr, OutputSection, OutputPhdr};
use super::mergedsection::MergedSection;

#[derive(Default)]
pub struct ContextArgs {
    pub Output:         String,
    pub Emulation:      MachineType,
    pub LIbraryPaths:   Vec<String>,
}

#[derive(Default)]
pub struct Context {
    pub Args:           ContextArgs,
    pub Objs:           Vec<Rc<RefCell<Objectfile>>>,
    /// holds all the collected files' `global` symbals here
    pub SymbolMap:      BTreeMap<String, Rc<RefCell<Symbol>>>,
    pub Buf:            Vec<u8>,
    pub MergedSections: Vec<Rc<RefCell<MergedSection>>>,
    // for output use
    pub Ehdr:           Box<OutputEhdr>,
    pub Shdr:           Box<OutputShdr>,
    pub Phdr:           Box<OutputPhdr>,
    pub Got:            Box<GotSection>,
    pub TpAddr:         u64,    // thread local pointer

    pub OutputSections: Vec<Rc<RefCell<OutputSection>>>,
    /// each chunk in this vector will finally be written into the target file.
    /// and these chunks have various types, including `chunk`, `outputshdr`, `outputehdr`, `outputsection`, 
    /// (all impl the trait `Chunker`). each has their own set of operations
    /// 
    /// note1:  ctx.OutputSections(also ctx.e/shdr) and just pointing to the same chunks
    /// whth some chunks in ctx.chunks. but Rc<Refcell> is difficult to use, 
    /// so i just use raw pointers here
    /// 
    /// note2: rust's built-in types `Rc` and `Refcell` has their own memory layout, which
    /// is not compatiable with c raw pointers. so we can't easily use std::ptr::addr_of to get the exact address
    /// &mut Rc<Refcell<T>> to *mut T:  &mut *T.as_ptr()
    pub Chunks:         Vec<*mut dyn Chunker>,
}

impl Context {
    pub fn new() -> Box<Context>{
        Box::new(Context { 
            Args: ContextArgs {
                Output: "a.out".into(),
                Emulation: MachineType::MachineTypeNone,
                LIbraryPaths: vec![],
            },
            ..default()
        })
    }
}
