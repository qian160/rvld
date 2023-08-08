#[derive(Default)]
#[allow(non_snake_case)]
pub struct Ehdr {
	pub Ident:      [u8; 16],
	pub Type:       u32,
	pub Machine:    u32,
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
/* 
#[derive(Default)]
#[allow(non_snake_case)]
pub struct Shdr{
	Name        :u32,
	Type        :u32,
	Flags       :u64,
	Addr        :u64,
	Offset      :u64,
	Size        :u64,
	Link        :u32,
	Info        :u32,
	AddrAlign   :u64,
	EntSize     :u64,
}
*/