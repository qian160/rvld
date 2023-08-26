//! useful informations collected and will be used during linking
use super::common::*;
use super::elf::{MachineType, Sym};
use super::sections::MergedSection;
use super::symbol::Symbol;
use super::output::{OutputEhdr, Chunker};

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
    /// holds all the `global` symbals here, which can be shared between files
    pub SymbolMap:      BTreeMap<String, Rc<RefCell<Symbol>>>,
    pub Buf:            Vec<u8>,
    pub MergedSections: Vec<Rc<RefCell<MergedSection>>>,
    pub Ehdr:           Box<OutputEhdr>,
    pub Chunks:         Vec<Box<dyn Chunker>>,
    pub InternalObj:    Box<Objectfile>,
    pub InternalEsyms:  Vec<Rc<Sym>>,
}

impl Context {
    pub fn new() -> Box<Context>{
        Box::new(Context { 
            Args: ContextArgs {
                Output: "a.out".into(),
                Emulation: MachineType::MachineTypeNone,
                LIbraryPaths: vec![],
            },
            ..Default::default()
            //Objs: vec![],
            //SymbolMap: BTreeMap::new(),
            //MergedSections: vec![]
        })
    }
}
