use super::elf::MachineType;

pub struct ContextArgs {
    pub Output:         String,
    pub Emulation:      MachineType,
    pub LIbraryPaths:   Vec<String>,
}

pub struct Context {
    pub Args: ContextArgs,
}

impl Context {
    pub fn new() -> Box<Context>{
        Box::new(Context { 
            Args: ContextArgs {
                Output: "a.out".to_string(),
                Emulation: MachineType::MachineTypeNone,
                LIbraryPaths: vec![]
            }
        })
    }
}

