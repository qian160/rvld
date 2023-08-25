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

// SHF_* define constants for the ELF Section Header's sh_flags field.
// Represented as Elf32_Word in Elf32_Ehdr and Elf64_Xword in Elf64_Ehdr which
// are both 4-byte and 8-byte unsigned integers, respectively.
// All of the constants are < 32-bits, so we use a u32 to represent these in order
// to make working with them easier.

/// Empty flags
pub const SHF_NONE: u64 = 0;
/// The section contains data that should be writable during process execution.
pub const SHF_WRITE: u64 = 1;
/// The section occupies memory during process execution. Some control sections
/// do not reside in the memory image of an object file; this attribute is off for
/// those sections.
pub const SHF_ALLOC: u64 = 1 << 1;
/// The section contains executable machine instructions.
pub const SHF_EXECINSTR: u64 = 1 << 2;
/// The data in the section may be merged to eliminate duplication. Unless the
/// SHF_STRINGS flag is also set, the data elements in the section are of a uniform size.
/// The size of each element is specified in the section header's sh_entsize field. If
/// the SHF_STRINGS flag is also set, the data elements consist of null-terminated
/// character strings. The size of each character is specified in the section header's
/// sh_entsize field.
///
/// Each element in the section is compared against other elements in sections with the
/// same name, type and flags. Elements that would have identical values at program
/// run-time may be merged. Relocations referencing elements of such sections must be
/// resolved to the merged locations of the referenced values. Note that any relocatable
/// values, including values that would result in run-time relocations, must be analyzed
/// to determine whether the run-time values would actually be identical. An
/// ABI-conforming object file may not depend on specific elements being merged, and an
/// ABI-conforming link editor may choose not to merge specific elements.
pub const SHF_MERGE: u64 = 1 << 4;
/// The data elements in the section consist of null-terminated character strings.
/// The size of each character is specified in the section header's sh_entsize field.
pub const SHF_STRINGS: u64 = 1 << 5;
/// The sh_info field of this section header holds a section header table index.
pub const SHF_INFO_LINK: u64 = 1 << 6;
/// This flag adds special ordering requirements for link editors. The requirements
/// apply if the sh_link field of this section's header references another section (the
/// linked-to section). If this section is combined with other sections in the output
/// file, it must appear in the same relative order with respect to those sections,
/// as the linked-to section appears with respect to sections the linked-to section is
/// combined with.
pub const SHF_LINK_ORDER: u64 = 1 << 7;
/// This section requires special OS-specific processing (beyond the standard linking
/// rules) to avoid incorrect behavior. If this section has either an sh_type value or
/// contains sh_flags bits in the OS-specific ranges for those fields, and a link
/// editor processing this section does not recognize those values, then the link editor
/// should reject the object file containing this section with an error.
pub const SHF_OS_NONCONFORMING: u64 = 1 << 8;
/// This section is a member (perhaps the only one) of a section group. The section must
/// be referenced by a section of type SHT_GROUP. The SHF_GROUP flag may be set only for
/// sections contained in relocatable objects (objects with the ELF header e_type member
/// set to ET_REL).
pub const SHF_GROUP: u64 = 1 << 9;
/// This section holds Thread-Local Storage, meaning that each separate execution flow
/// has its own distinct instance of this data. Implementations need not support this flag.
pub const SHF_TLS: u64 = 1 << 10;
/// This flag identifies a section containing compressed data. SHF_COMPRESSED applies only
/// to non-allocable sections, and cannot be used in conjunction with SHF_ALLOC. In
/// addition, SHF_COMPRESSED cannot be applied to sections of type SHT_NOBITS.
///
/// All relocations to a compressed section specifiy offsets to the uncompressed section
/// data. It is therefore necessary to decompress the section data before relocations can
/// be applied. Each compressed section specifies the algorithm independently. It is
/// permissible for different sections in a given ELF object to employ different
/// compression algorithms.
///
/// Compressed sections begin with a compression header structure that identifies the
/// compression algorithm.
pub const SHF_COMPRESSED: u64 = 1 << 11;
/// Masked bits are reserved for operating system-specific semantics.
pub const SHF_MASKOS: u64 = 0x0ff00000;
/// Masked bits are reserved for processor-specific semantics.
pub const SHF_MASKPROC: u64 = 0xf0000000;
