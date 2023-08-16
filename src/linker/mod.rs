pub mod elf;
pub mod file;
pub mod context;

//use elf::*;
pub use self::elf::{EHDR_SIZE, ElfGetName};