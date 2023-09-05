use super::common::*;
use super::output::{Chunk, GetOutputName};
use super::inputsections::SectionFragment;

#[derive(Default,Debug)]
pub struct MergedSection {
	pub Chunk:	Chunk,
	/// note: key is not always strings
	pub Map:	BTreeMap<String, Rc<RefCell<SectionFragment>>>,
}

impl Deref for MergedSection {
	type Target = Chunk;
	fn deref(&self) -> &Self::Target {
		&self.Chunk
	}
}

impl DerefMut for MergedSection {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.Chunk
	}
}

impl MergedSection {
	pub fn new(name: &str, flags: u64, ty: u32) -> Rc<RefCell<MergedSection>> {
		let mut m = MergedSection {
			Chunk: Chunk::new(),
			..default()
		};

		m.Name = name.into();
		m.Shdr.Flags = flags;
		m.Shdr.Type = ty;

		m.ToRcRefcell()
	}

	pub fn GetInstance(ctx: &mut Context, name: &str, ty: u32, flags: u64) -> Rc<RefCell<Self>> {
		let name = GetOutputName(name, flags);
		// ignore these flags
		let flags = flags &
			!abi::SHF_GROUP as u64 & !abi::SHF_MERGE as u64 &
			!abi::SHF_STRINGS as u64 & !abi::SHF_COMPRESSED as u64;

		let osec = ctx.MergedSections.iter().find(
			|osec| {
				let osec = osec.borrow();
				name == osec.Name && flags == osec.Shdr.Flags && ty == osec.Shdr.Type
			}
		);

		match osec {
			Some(o) => o.clone(),
			None => {
				let osec = MergedSection::new(&name, flags, ty);
				ctx.MergedSections.push(osec.clone());
				osec
			}
		}
	}

	pub fn Insert(m: Rc<RefCell<Self>>, key: String, p2align: u8) -> Rc<RefCell<SectionFragment>> {
		let mut ms = m.borrow_mut();

		let frag = match ms.Map.get(&key) {
			Some(f) => f,
			None => {
				ms.Map.insert(key.clone(), SectionFragment::new(m.clone()));
				ms.Map.get(&key).unwrap()
			}
		};

		let p2align_old = frag.borrow().P2Align;
		frag.borrow_mut().P2Align = p2align_old.max(p2align);
		frag.clone()
	}
	// sort?
	pub fn AssignOffsets(&mut self) {
		struct Fragment<'a> {
			pub Key: String,
			pub val: &'a mut SectionFragment,
		}
		let mut fragments: Vec<Fragment> = vec![];
		for (key, frag) in &mut self.Map {
			let ptr = ptr2ref(frag.as_ptr());
			fragments.push(Fragment { Key: key.clone(), val: ptr })
		}

		fragments.sort_by(|x, y| {
			if x.val.P2Align != y.val.P2Align {
				return x.val.P2Align.cmp(&y.val.P2Align);
			}
			if x.Key.len() != y.Key.len() {
				return x.Key.len().cmp(&y.Key.len());
			}
			return x.Key.cmp(&y.Key);
		});

		let mut offset = 0;
		let mut p2align = 0;
		for mut frag in fragments {
			offset = AlignTo(offset, 1 << frag.val.P2Align);
			frag.val.Offset = offset as u32;
			offset += frag.Key.len();
			p2align = p2align.max(frag.val.P2Align);
		}

//		let mut offset = 0;
//		let mut p2align = 0;
//		for (key, frag) in &mut self.Map {
//			offset = AlignTo(offset, 1 << frag.P2Align);
//			frag.Offset = offset as u32;
//			offset += key.len();
//			p2align = p2align.max(frag.P2Align);
//		}
		self.Shdr.Size = AlignTo(offset, 1 << p2align);
		self.Shdr.AddrAlign = 1 << p2align;
	}

}
