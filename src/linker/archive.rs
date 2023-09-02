use super::common::*;
use crate::linker::elf::FileType;
use super::file::File;

pub const AR_IDENT: &[u8] = b"!<arch>\n";
pub const ARHDR_SIZE: usize = std::mem::size_of::<ArHdr>();

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
// size = [49, 50, 51, 32, 32, 32, 32, 32, 32, 32] => "123       " => parse -> 123

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

	pub fn ReadName(&self, strtab: &[u8]) -> String {
		// long filename
		if self.Name.starts_with(b"/") {
			let start = atoi(&self.Name[1..]);
			let end = start + strtab.windows(2).position(|w| w ==  b"/\n").unwrap();
			//return String::from_utf8(strtab[start..end].into()).unwrap().into();
			return unsafe { String::from_utf8_unchecked(strtab[start..end].into())};
		}
		// short filename
		let end = self.Name.iter().position( |&x| x == b'/').unwrap();
		unsafe { String::from_utf8_unchecked(self.Name[..end].into()) }
	}
}


pub fn ReadArchiveMembers(file: Rc<File>) -> Vec<Rc<File>>{
	assert!(file.Type == FileType::FileTypeArchive);

	let mut pos = 8;	
	let mut strTab = ByteSequence::default();
	let mut files: Vec<Rc<File>> = vec![];
	let contents = &file.Contents;
	let len = contents.len();
	while len - pos > 1 {
		if pos % 2 == 1 {
			pos = pos + 1;
		}
		let hdr = Read::<ArHdr>(&contents[pos..]);
		assert_eq!(hdr.Fmag, [0x60, 0xa]);
		let dataStart = pos + ARHDR_SIZE;
		pos = dataStart + hdr.GetSize();

		let contents = &contents[dataStart..pos];

		if hdr.IsSymtab() {
			continue;
		}
		else if hdr.IsStrtab() {
			strTab = ByteSequence::new(contents.as_ptr(), contents.len());
			continue;
		}

		let f = File::new(
			&hdr.ReadName(strTab.GetSlice()),
			Some(contents.into()),
			Some(file.clone())
		);
		files.push(f);
	}
	
//	crate::warn!("#{} objs collected from {}",files.len(), file.Name);
	files
}