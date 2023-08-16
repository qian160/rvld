use super::{elf::MachineType, file::Objectfile};

pub struct ContextArgs {
    pub Output:         String,
    pub Emulation:      MachineType,
    pub LIbraryPaths:   Vec<String>,
}

pub struct Context {
    pub Args:   ContextArgs,
    pub Objs:   Vec<Box<Objectfile>>
}

impl Context {
    pub fn new() -> Box<Context>{
        Box::new(Context { 
            Args: ContextArgs {
                Output: "a.out".to_string(),
                Emulation: MachineType::MachineTypeNone,
                LIbraryPaths: vec![],
            },
            Objs: vec![],
        })
    }
}

