//! useful informations collected and will be used during linking
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::BTreeMap;
use super::elf::MachineType;
use super::objectfile::Objectfile;
use super::symbol::Symbol;

pub struct ContextArgs {
    pub Output:         String,
    pub Emulation:      MachineType,
    pub LIbraryPaths:   Vec<String>,
}

pub struct Context {
    pub Args:       ContextArgs,
    pub Objs:       Vec<Rc<RefCell<Objectfile>>>,
    /// holds all the `global` symbals here, which can be shared between files
    pub SymbolMap:  BTreeMap<String, Rc<RefCell<Symbol>>>,
}

impl Context {
    pub fn new() -> Box<Context>{
        Box::new(Context { 
            Args: ContextArgs {
                Output: "a.out".into(),
                Emulation: MachineType::MachineTypeNone,
                LIbraryPaths: vec![],
            },
            Objs: vec![],
            SymbolMap: BTreeMap::new(),
        })
    }
}
