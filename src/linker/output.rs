use super::elf::Shdr;
use super::abi;

#[derive(Default,Debug)]
pub struct Chunk {
    pub Name:   String,
    pub Shdr:   Shdr,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            Shdr: Shdr{
				AddrAlign: 1,
				..Default::default()
			},
            Name: "".into()
        }
    }
    
}

pub const PREFIXES: [&str; 13] = [
	".text.", ".data.rel.ro.", ".data.", ".rodata.", ".bss.rel.ro.", ".bss.",
	".init_array.", ".fini_array.", ".tbss.", ".tdata.", ".gcc_except_table.",
	".ctors.", ".dtors.",
];

pub fn GetOutputName(name: &str, flags: u64) -> String {
	if (name == ".rodata" || name.starts_with(".rodata.")) &&
	flags & abi::SHF_MERGE != 0 {
		return if flags & abi::SHF_STRINGS != 0 {
			".rodata.str".into()
		}
		else {
			".rodata.cst".into()	// const
		};
	}

	for prefix in PREFIXES {
		let stem = &prefix[..prefix.len() - 1];	// remove the last '.'
		if name == stem || name.starts_with(prefix) {
			return stem.into();
		}
	}

	name.into()

}