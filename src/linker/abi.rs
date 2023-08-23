#![allow(unused)]
// SHT_* define constants for the ELF Section Header's Type field.
// Represented as Elf32_Word in Elf32_Ehdr and Elf64_Word in Elf64_Ehdr which
// are both 4-byte unsigned integers with 4-byte alignment

/// Inactive section with undefined values
pub const SHT_NULL: u32 = 0;
/// Information defined by the program, includes executable code and data
pub const SHT_PROGBITS: u32 = 1;
/// Section data contains a symbol table
pub const SHT_SYMTAB: u32 = 2;
/// Section data contains a string table
pub const SHT_STRTAB: u32 = 3;
/// Section data contains relocation entries with explicit addends
pub const SHT_RELA: u32 = 4;
/// Section data contains a symbol hash table. Must be present for dynamic linking
pub const SHT_HASH: u32 = 5;
/// Section data contains information for dynamic linking
pub const SHT_DYNAMIC: u32 = 6;
/// Section data contains information that marks the file in some way
pub const SHT_NOTE: u32 = 7;
/// Section data occupies no space in the file but otherwise resembles SHT_PROGBITS
pub const SHT_NOBITS: u32 = 8;
/// Section data contains relocation entries without explicit addends
pub const SHT_REL: u32 = 9;
/// Section is reserved but has unspecified semantics
pub const SHT_SHLIB: u32 = 10;
/// Section data contains a minimal set of dynamic linking symbols
pub const SHT_DYNSYM: u32 = 11;
/// Section data contains an array of constructors
pub const SHT_INIT_ARRAY: u32 = 14;
/// Section data contains an array of destructors
pub const SHT_FINI_ARRAY: u32 = 15;
/// Section data contains an array of pre-constructors
pub const SHT_PREINIT_ARRAY: u32 = 16;
/// Section group
pub const SHT_GROUP: u32 = 17;
/// Extended symbol table section index
pub const SHT_SYMTAB_SHNDX: u32 = 18;
/// Values in [SHT_LOOS, SHT_HIOS] are reserved for operating system-specific semantics.
pub const SHT_LOOS: u32 = 0x60000000;
/// Object attributes
pub const SHT_GNU_ATTRIBUTES: u32 = 0x6ffffff5;
/// GNU-style hash section
pub const SHT_GNU_HASH: u32 = 0x6ffffff6;
/// Pre-link library list
pub const SHT_GNU_LIBLIST: u32 = 0x6ffffff7;
/// Version definition section
pub const SHT_GNU_VERDEF: u32 = 0x6ffffffd;
/// Version needs section
pub const SHT_GNU_VERNEED: u32 = 0x6ffffffe;
/// Version symbol table
pub const SHT_GNU_VERSYM: u32 = 0x6fffffff;
/// Values in [SHT_LOOS, SHT_HIOS] are reserved for operating system-specific semantics.
pub const SHT_HIOS: u32 = 0x6fffffff;
/// Values in [SHT_LOPROC, SHT_HIPROC] are reserved for processor-specific semantics.
pub const SHT_LOPROC: u32 = 0x70000000;
/// IA_64 extension bits
pub const SHT_IA_64_EXT: u32 = 0x70000000; // SHT_LOPROC + 0;
/// IA_64 unwind section
pub const SHT_IA_64_UNWIND: u32 = 0x70000001; // SHT_LOPROC + 1;
/// Values in [SHT_LOPROC, SHT_HIPROC] are reserved for processor-specific semantics.
pub const SHT_HIPROC: u32 = 0x7fffffff;
/// Values in [SHT_LOUSER, SHT_HIUSER] are reserved for application-specific semantics.
pub const SHT_LOUSER: u32 = 0x80000000;
/// Values in [SHT_LOUSER, SHT_HIUSER] are reserved for application-specific semantics.
pub const SHT_HIUSER: u32 = 0x8fffffff;

/// special section indexes

/// This value marks an undefined, missing, irrelevant, or otherwise meaningless
/// section reference.
pub const SHN_UNDEF: u16 = 0;
/// Symbols with st_shndx=SHN_ABS are absolute and are not affected by relocation.
pub const SHN_ABS: u16 = 0xfff1;
/// Symbols with st_shndx=SHN_COMMON are sometimes used for unallocated C external variables.
pub const SHN_COMMON: u16 = 0xfff2;
pub const SHN_XINDEX: u16 = 0xffff;

// ET_* define constants for the ELF File Header's Type field.
// Represented as Elf32_Half in Elf32_Ehdr and Elf64_Half in Elf64_Ehdr which
// are both are 2-byte unsigned integers with 2-byte alignment

/// No file type
pub const ET_NONE: u16 = 0;
/// Relocatable file
pub const ET_REL: u16 = 1;
/// Executable file
pub const ET_EXEC: u16 = 2;
/// Shared object file
pub const ET_DYN: u16 = 3;
/// Core file
pub const ET_CORE: u16 = 4;
/// Operating system-specific
pub const ET_LOOS: u16 = 0xfe00;
/// Operating system-specific
pub const ET_HIOS: u16 = 0xfeff;
/// Processor-specific
pub const ET_LOPROC: u16 = 0xff00;
/// Processor-specific
pub const ET_HIPROC: u16 = 0xffff;

// EM_* define constants for the ELF File Header's Machine field.
// Represented as Elf32_Half in Elf32_Ehdr and Elf64_Half in Elf64_Ehdr which
// are both 2-byte unsigned integers with 2-byte alignment

/// RISC-V
pub const EM_RISCV: u16 = 243;

// ELFCLASS* define constants for Ident[4] in Ehdr

/// Invalid ELF file class
pub const ELFCLASSNONE: u8 = 0;
/// 32-bit ELF file
pub const ELFCLASS32: u8 = 1;
/// 64-bit ELF file
pub const ELFCLASS64: u8 = 2;