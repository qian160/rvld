pub mod elf;
pub mod file;
pub mod inputfile;
pub mod objectfile;

//use elf::*;
pub use self::elf::{EHDR_SIZE, ElfGetName};