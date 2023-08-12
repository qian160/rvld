pub const EHDR_SIZE: usize = core::mem::size_of::<Ehdr>();
pub const SHDR_SIZE: usize = core::mem::size_of::<Shdr>();
pub const SYM_SIZE: usize = core::mem::size_of::<Sym>();
const MAGIC: &[u8] = "\x7fELF".as_bytes();

pub fn checkMagic(s: &Vec<u8>) -> bool {
    s.starts_with(MAGIC)
}

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

impl std::fmt::Debug for Ehdr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "
Ident:		{:x?}
Type:		{:?}
Machine:	{:?}
Version:	{:?}
Entry:		{:?}
PhOff:		{:?}
ShOff:		{:?}
Flags:		{:?}
EhSize:		{:?}
PhEntSize:	{:?}
PhNum:		{:?}
ShEntSize:	{:?}
ShNum:		{:?}
ShStrndx:	{:?}
		", self.Ident, self.Type, self.Machine, self.Version, self.Entry, self.PhOff, self.ShOff, self.Flags, self.EhSize, self.PhEntSize, self.PhNum, self.ShEntSize, self.ShNum, self.ShStrndx)
	}
}

impl std::fmt::Debug for Shdr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "
Name:		{}
Type:		{}
Flags:		{}
Addr:		{}
Offset:		{}
Size:		{}
Link:		{}
Info:		{}
AddrAlign:	{}
EntSize:	{}
		", self.Name, self.Type, self.Flags, self.Addr, self.Offset, self.Size, self.Link, self.Info, self.AddrAlign, self.EntSize)
	}
}

impl std::fmt::Debug for Sym {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "
Name:		{:x?}
Info:		{:?}
Other:		{:?}
Shndx:		{:?}
Val:		{:?}
Size:		{:?}
		", self.Name, self.Info, self.Other, self.Shndx, self.Val, self.Size)
	}
}

pub fn ElfGetName(strtab: &Vec<u8>, offset: usize) -> String {
    let length = strtab[offset..].iter().position(|&x| x == 0).unwrap();
    std::str::from_utf8(
        &strtab[offset..offset+length]
        .to_vec()).unwrap().to_string()
}