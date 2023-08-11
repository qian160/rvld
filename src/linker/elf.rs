pub const EhdrSize: usize = core::mem::size_of::<Ehdr>();
pub const ShdrSize: usize = core::mem::size_of::<Shdr>();

#[derive(Default, Debug)]
#[allow(non_snake_case)]
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

#[derive(Default, Debug)]
#[allow(non_snake_case)]
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
