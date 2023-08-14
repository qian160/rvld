pub const EHDR_SIZE: usize = core::mem::size_of::<Ehdr>();
pub const SHDR_SIZE: usize = core::mem::size_of::<Shdr>();
pub const SYM_SIZE: usize = core::mem::size_of::<Sym>();

const MAGIC: &[u8] = "\x7fELF".as_bytes();
pub fn checkMagic(s: &Vec<u8>) -> bool {
    s.starts_with(MAGIC)
}

use super::inputfile::InputFile;

#[derive(Default, Clone)]
#[allow(non_snake_case)]
#[repr(C)]
pub struct Ehdr {
	pub Ident:      [u8; 16],
	pub Type:       u16,
	pub Machine:    u16,
	pub Version:    u32,
	pub Entry:      u64,
	pub PhOff:      u64,
	pub ShOff:      u64,
	pub Flags:      u32,
	pub EhSize:     u16,
	pub PhEntSize:  u16,
	pub PhNum:      u16,
	pub ShEntSize:  u16,
	pub ShNum:      u16,
	pub ShStrndx:   u16,
}

#[derive(Default, Clone)]
#[allow(non_snake_case)]
#[repr(C)]
pub struct Shdr{
	pub Name:       u32,
	pub Type:       u32,
	pub Flags:      u64,
	pub Addr:       u64,
	pub Offset:     u64,
	pub Size:       u64,
	pub Link:       u32,
	pub Info:       u32,
	pub AddrAlign:  u64,
	pub EntSize:    u64,
}

#[derive(Default)]
#[allow(non_snake_case)]
#[repr(C)]
pub struct Sym {
	pub Name:       u32,
	pub Info:       u8,
	pub Other:      u8,
	pub Shndx:      u16,
	pub Val:        u64,
	pub Size:       u64,
}


pub enum FileType{
	FileTypeUnknown,
	FileTypeObject,
}

#[derive(Debug)]
pub enum MachineType {
	MachineTypeNone,
	MachineTypeRISCV64,
}

impl MachineType {
	pub fn String(&self) -> String {
		match self {
			MachineType::MachineTypeRISCV64 =>
				"riscv64".to_string(),
			_ =>
				"unknown".to_string()
		}
	}
}

pub fn GetFileType(inputfile: &InputFile) -> FileType {
	let et = inputfile.Ehdr.Type;
	match et {
		elf::abi::ET_REL => 
			FileType::FileTypeObject,
		_ =>
			FileType::FileTypeUnknown
	}
}

pub fn GetMachineType(inputfile: &InputFile) -> MachineType {
	let ft = GetFileType(inputfile);
	let ehdr = &inputfile.Ehdr;
	match ft {
		FileType::FileTypeObject => {
			let mt = inputfile.Ehdr.Machine;
			if mt == elf::abi::EM_RISCV {
				let class = ehdr.Ident[4];
				return match class {
					elf::abi::ELFCLASS64 => MachineType::MachineTypeRISCV64,
					_ => MachineType::MachineTypeNone
				};
			};
			MachineType::MachineTypeNone
		}
		_ =>
			MachineType::MachineTypeNone
	}
}

pub fn ElfGetName(strtab: &Vec<u8>, offset: usize) -> String {
    let length = strtab[offset..].iter().position(|&x| x == 0).unwrap();
    std::str::from_utf8(
        &strtab[offset..offset+length]
        .to_vec()).unwrap().to_string()
}