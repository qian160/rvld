pub mod elf;
pub mod file;
pub mod context;
pub mod archive;
pub mod sections;
pub mod symbol;
pub mod passes;
pub mod objectfile;

//use elf::*;
pub use self::elf::{EHDR_SIZE, ElfGetName};