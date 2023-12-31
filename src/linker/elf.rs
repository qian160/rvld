#![allow(non_snake_case)]
pub const EHDR_SIZE: usize = core::mem::size_of::<Ehdr>();
pub const SHDR_SIZE: usize = core::mem::size_of::<Shdr>();
pub const PHDR_SIZE: usize = core::mem::size_of::<Phdr>();

pub const IMAGE_BASE: usize = 0x200000;
pub const MAGIC: &[u8] = b"\x7fELF";
pub const PAGESIZE: u64 = 4096;

pub fn checkMagic(s: &[u8]) -> bool {
    s.starts_with(MAGIC)
}

use crate::utils::Read;

use super::file::File;
use super::context::Context;

use elf::abi;

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
	pub Offset:     usize,
    /// Section size in bytes
	pub Size:       usize,
    /// Defined by section type
	pub Link:       u32,
    /// Defined by section type
	pub Info:       u32,
    /// address alignment
	pub AddrAlign:  u64,
    /// size of an entry if section data is an array of entries
	pub EntSize:    usize,
}

#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct Phdr {
	/// Program segment type
    pub Type:		u32,
    /// Flags for this segment
    pub Flags:		u32,
    /// Offset into the ELF file where this segment begins
    pub Offset:		u64,
    /// Virtual adress where this segment should be loaded
    pub VAddr:		u64,
    /// Physical address where this segment should be loaded
    pub PAddr:		u64,
    /// Size of this segment in the file
    pub FileSize:	u64,
    /// Size of this segment in memory
    pub MemSize:	u64,
    /// file and memory alignment
    pub Align:		u64,
}

#[derive(Default)]
#[repr(C)]
pub struct Sym {
    /// This member holds an index into the symbol table's string table,
    /// which holds the character representations of the symbol names. If the
    /// value is non-zero, it represents a string table index that gives the
    /// symbol name. Otherwise, the symbol table entry has no name.
	pub Name:       u32,
    /// This member specifies the symbol's type and binding attributes. each 4 bits
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

#[derive(Debug, PartialEq, Default)]
pub enum MachineType {
	#[default]
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
		self.Shndx == abi::SHN_ABS
	}
	/// This value marks an undefined, missing, irrelevant, or otherwise meaningless section reference.
	pub fn IsUndef(&self) -> bool {
		self.Shndx == abi::SHN_UNDEF
	}
	/// Symbols with st_shndx=SHN_COMMON are sometimes used for unallocated C external variables.
	pub fn IsCommon(&self) -> bool {
		self.Shndx == abi::SHN_COMMON
	}
    /// little endian
    pub fn Type(&self) -> u8 {
        self.Info & 0b1111
    }
}

pub fn GetMachineType(file: &File) -> MachineType {
	let ft = &file.Type;
	let Contents = &file.Contents;
	let machine = Read::<u16>(&Contents[18..]);
	match ft {
		FileType::FileTypeObject => {
			if machine == abi::EM_RISCV {
				return match Contents[4]{
					abi::ELFCLASS64 => MachineType::MachineTypeRISCV64,
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
pub fn ElfGetName(strtab: &[u8], offset: usize) -> String {
    let length = strtab[offset..].iter().position(|&x| x == 0).unwrap();
    unsafe { std::string::String::from_utf8_unchecked(
		(strtab[offset..offset+length]).to_vec())}
}

pub fn CheckFileCompatibility(ctx: &Context, file: &File) {
	let mt = GetMachineType(&file);
	if mt != ctx.Args.Emulation {
		crate::error!("{}: incompatible file type!", file.Name);
	}
}