#![allow(non_snake_case)]
pub const EHDR_SIZE: usize = core::mem::size_of::<Ehdr>();
pub const SHDR_SIZE: usize = core::mem::size_of::<Shdr>();
pub const SYM_SIZE: usize = core::mem::size_of::<Sym>();

const MAGIC: &[u8] = b"\x7fELF";

pub fn checkMagic(s: &Vec<u8>) -> bool {
    s.starts_with(MAGIC)
}

use crate::utils::Read;

use super::file::File;
use super::context::Context;

/// The ELF File Header starts off every ELF file and both identifies the
/// file contents and informs how to interpret said contents. This includes
/// the width of certain fields (32-bit vs 64-bit), the data endianness, the
/// file type, and more.
#[derive(Default, Clone)]
#[repr(C)]
pub struct Ehdr {
	pub Ident:      [u8; 16],
	/// ELF file type
	pub Type:       u16,
	/// Target machine architecture
	pub Machine:    u16,
    /// elf version
	pub Version:    u32,
    /// Virtual address of program entry point
    /// This member gives the virtual address to which the system first transfers control,
    /// thus starting the process. If the file has no associated entry point, this member holds zero.
    ///
    /// Note: Type is Elf32_Addr or Elf64_Addr which are either 4 or 8 bytes. We aren't trying to zero-copy
    /// parse the FileHeader since there's only one per file and its only ~45 bytes anyway, so we use
    /// u64 for the three Elf*_Addr and Elf*_Off fields here.
	pub Entry:      u64,
    /// This member holds the program header table's file offset in bytes. If the file has no program header
    /// table, this member holds zero.
	pub PhOff:      u64,
    /// This member holds the section header table's file offset in bytes. If the file has no section header
    /// table, this member holds zero.
	pub ShOff:      u64,
    /// This member holds processor-specific flags associated with the file. Flag names take the form EF_machine_flag.
	pub Flags:      u32,
    /// This member holds the ELF header's size in bytes.
	pub EhSize:     u16,
    /// This member holds the size in bytes of one entry in the file's program header table; all entries are the same size.
	pub PhEntSize:  u16,
    /// This member holds the number of entries in the program header table. Thus the product of e_phentsize and e_phnum
    /// gives the table's size in bytes. If a file has no program header table, e_phnum holds the value zero.
	pub PhNum:      u16,
    /// This member holds a section header's size in bytes. A section header is one entry in the section header table;
    /// all entries are the same size.
	pub ShEntSize:  u16,
    /// This member holds the number of entries in the section header table. Thus the product of e_shentsize and e_shnum
    /// gives the section header table's size in bytes. If a file has no section header table, e_shnum holds the value zero.
    ///
    /// If the number of sections is greater than or equal to SHN_LORESERVE (0xff00), this member has the value zero and
    /// the actual number of section header table entries is contained in the sh_size field of the section header at index 0.
    /// (Otherwise, the sh_size member of the initial entry contains 0.)
	pub ShNum:      u16,
    /// This member holds the section header table index of the entry associated with the section name string table. If the
    /// file has no section name string table, this member holds the value SHN_UNDEF.
    ///
    /// If the section name string table section index is greater than or equal to SHN_LORESERVE (0xff00), this member has
    /// the value SHN_XINDEX (0xffff) and the actual index of the section name string table section is contained in the
    /// sh_link field of the section header at index 0. (Otherwise, the sh_link member of the initial entry contains 0.)
	pub ShStrndx:   u16,
}

#[derive(Default, Clone)]
#[repr(C)]
pub struct Shdr{
	/// Section Name
	pub Name:       u32,
    /// Section Type
	pub Type:       u32,
    /// Section Flags
	pub Flags:      u64,
    /// in-memory address where this section is loaded
	pub Addr:       u64,
    /// Byte-offset into the file where this section starts
	pub Offset:     u64,
    /// Section size in bytes
	pub Size:       u64,
    /// Defined by section type
	pub Link:       u32,
    /// Defined by section type
	pub Info:       u32,
    /// address alignment
	pub AddrAlign:  u64,
    /// size of an entry if section data is an array of entries
	pub EntSize:    u64,
}

#[derive(Default)]
#[repr(C)]
pub struct Sym {
    /// This member holds an index into the symbol table's string table,
    /// which holds the character representations of the symbol names. If the
    /// value is non-zero, it represents a string table index that gives the
    /// symbol name. Otherwise, the symbol table entry has no name.
	pub Name:       u32,
    /// This member specifies the symbol's type and binding attributes.
	pub Info:       u8,
	/// This member currently specifies a symbol's visibility.
	pub Other:      u8,
    /// Every symbol table entry is defined in relation to some section. This
    /// member holds the relevant section header table index. As the sh_link and
    /// sh_info interpretation table and the related text describe, some section
    /// indexes indicate special meanings.
    ///
    /// If this member contains SHN_XINDEX, then the actual section header index
    /// is too large to fit in this field. The actual value is contained in the
    /// associated section of type SHT_SYMTAB_SHNDX.
	pub Shndx:      u16,
    /// This member gives the value of the associated symbol. Depending on the
    /// context, this may be an absolute value, an address, and so on.
    ///
    /// * In relocatable files, st_value holds alignment constraints for a
    ///   symbol whose section index is SHN_COMMON.
    /// * In relocatable files, st_value holds a section offset for a defined
    ///   symbol. st_value is an offset from the beginning of the section that
    ///   st_shndx identifies.
    /// * In executable and shared object files, st_value holds a virtual
    ///   address. To make these files' symbols more useful for the dynamic
    ///   linker, the section offset (file interpretation) gives way to a
    ///   virtual address (memory interpretation) for which the section number
    ///   is irrelevant.
	pub Val:        u64,
    /// This member gives the symbol's size.
    /// For example, a data object's size is the number of bytes contained in
    /// the object. This member holds 0 if the symbol has no size or an unknown
    /// size.
	pub Size:       u64,
}

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
				"riscv64".into(),
			_ =>
				"unknown".into()
		}
	}
}

impl Sym {
	/// some special shndx values
	
	/// Symbols with st_shndx=SHN_ABS are absolute and are not affected by relocation.
	pub fn IsAbs(&self) -> bool {
		self.Shndx == super::abi::SHN_ABS
	}
	/// This value marks an undefined, missing, irrelevant, or otherwise meaningless section reference.
	pub fn IsUndef(&self) -> bool {
		self.Shndx == super::abi::SHN_UNDEF
	}
	/// Symbols with st_shndx=SHN_COMMON are sometimes used for unallocated C external variables.
	pub fn IsCommon(&self) -> bool {
		self.Shndx == super::abi::SHN_COMMON
	}
}

pub fn GetMachineType(file: &File) -> MachineType {
	let ft = &file.Type;
	let Contents = &file.Contents;
	let machine = Read::<u16>(&Contents[18..]).unwrap();
	match ft {
		FileType::FileTypeObject => {
			if machine == super::abi::EM_RISCV {
				return match Contents[4]{
					super::abi::ELFCLASS64 => MachineType::MachineTypeRISCV64,
					_ => MachineType::MachineTypeNone
				};
			};
			MachineType::MachineTypeNone
		}
		_ =>
			MachineType::MachineTypeNone
	}
}

#[allow(unused)]
pub fn ElfGetName(strtab: &Vec<u8>, offset: usize) -> String {
    let length = strtab[offset..].iter().position(|&x| x == 0).unwrap();
    std::str::from_utf8(
		&strtab[offset..offset+length]).unwrap().into()
}

pub fn CheckFileCompatibility(ctx: &Context, file: &File) {
	let mt = GetMachineType(&file);
	if mt != ctx.Args.Emulation {
		crate::error!("{}: incompatible file type!", file.Name);
	}
}