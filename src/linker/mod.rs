pub mod elf;
mod magic;
pub mod file;
pub mod inputfile;

//use elf::*;
pub use magic::checkMagic;
pub use elf::EhdrSize;