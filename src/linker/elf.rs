pub const EHDR_SIZE: usize = core::mem::size_of::<Ehdr>();
pub const SHDR_SIZE: usize = core::mem::size_of::<Shdr>();
pub const SYM_SIZE: usize = core::mem::size_of::<Sym>();
pub const ARHDR_SIZE: usize = std::mem::size_of::<ArHdr>();

const MAGIC: &[u8] = "\x7fELF".as_bytes();
pub const AR_IDENT: &[u8] = b"!<arch>\n";

pub fn checkMagic(s: &Vec<u8>) -> bool {
    s.starts_with(MAGIC)
}

use crate::utils::Read;

use super::file::File;
use super::context::Context;
use std::path::Path;

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


#[derive(Default)]
#[allow(non_snake_case)]
#[repr(C)]
// [!<arch>\n](8 bytes) [ArHdr1] [ obj1 ] [ArHdr2] [    obj2    ] [ArHdr3] [  obj3  ] ...
pub struct ArHdr{
	// note: name may have a prefix '/' to specify
	// wheter it will use a short or long filename
	pub Name:   [u8; 16],
	pub Date:   [u8; 12],
	pub Uid:    [u8; 6],
	pub Gid:    [u8; 6],
	pub Mode:   [u8; 8],
	pub Size:   [u8; 10],
	pub Fmag:   [u8; 2],
}
// size: decimal(not binary) encoding using string,
// e.g:
// size = [48, 32, 32, 32, 32, 32, 32, 32, 49, 52] => "0       14" => parse -> 0


#[derive(PartialEq, Default, Clone, Debug)]
pub enum FileType{
	#[default]
	FileTypeUnknown,
	FileTypeEmpty,
	FileTypeObject,
	FileTypeArchive,
}

#[derive(Debug, PartialEq)]
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

pub fn atoi(s: &[u8]) -> usize {
	let s = std::str::from_utf8(s).unwrap().trim();
	let end = s.trim().find(" ").unwrap_or(s.len());
	s[0..end].parse::<usize>().unwrap()
}

impl ArHdr {
	pub fn IsStrtab(&self) -> bool {
		self.Name.starts_with(b"// ")
	}

	pub fn IsSymtab(&self) -> bool {
		self.Name.starts_with(b"/ ") ||
		self.Name.starts_with(b"/SYM64/ ")
	}

	pub fn GetSize(&self) -> usize {
		atoi(&self.Size)
	}

	pub fn ReadName(&self, strtab: &Vec<u8>) -> String {
		// long filename
		if self.Name.starts_with(b"/") {
			let start = atoi(&self.Name[1..]);
			let end = start + strtab.windows(2).position(|w| w ==  b"/\n").unwrap();
			return String::from_utf8(strtab[start..end].into()).unwrap().to_string();
		}
		// short filename
		let end = self.Name.iter().position( |&x| x == b'/').unwrap();
		String::from_utf8(self.Name[..end].into()).unwrap().to_string()
	}
}

pub fn GetMachineType(file: &File) -> MachineType {
	let ft = &file.Type;
	let Contents = std::fs::read(&file.Name).unwrap();
	let machine = Read::<u16>(&Contents[18..]).unwrap();
	match ft {
		FileType::FileTypeObject => {
			if machine == elf::abi::EM_RISCV {
				return match Contents[4]{
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

pub fn OpenLibrary(path: &str) -> Option<Box<File>> {
	match Path::exists(Path::new(path)) {
		true  => Some(File::new(path, None)),
		false => None 
	}
}

pub fn FindLibrary(ctx: &Context, name: &str) -> Option<Box<File>> {
	for dir in &ctx.Args.LIbraryPaths {
		let stem = dir.to_owned() + "/lib" + name + ".a";
		let f = OpenLibrary(&stem);
		if f.is_some() {
			return Some(f.unwrap());
		}
	}
	None
}

pub fn ReadArchiveMembers(file: &File) -> Vec<Box<File>>{
	assert!(file.Type == FileType::FileTypeArchive);

	let mut pos = 8;	
	let mut strTab: Vec<u8> = vec![];
	let mut files: Vec<Box<File>> = vec![];
	let Contents = std::fs::read(&file.Name).unwrap();
	let len = Contents.len();
	while len - pos > 1 {
		if pos % 2 == 1 {
			pos = pos + 1;
		}
		let hdr = Read::<ArHdr>(&Contents[pos..]).unwrap();
		assert_eq!(hdr.Fmag, [0x60, 0xa]);
		let dataStart = pos + ARHDR_SIZE;
		pos = dataStart + hdr.GetSize();

		let contents = &Contents[dataStart..pos];

		if hdr.IsSymtab() {
			continue;
		}
		else if hdr.IsStrtab() {
			strTab = contents.into();
			continue;
		}
		let name = hdr.ReadName(&strTab);
		//crate::debug!("{}", name);
		let mut f = File::new(&name, Some(contents.into()));
		f.Parent = Some(Box::new(file.clone()));
		files.push(f);
	}
	files
}