use std::{mem::size_of, rc::Rc, cell::RefCell};
use crate::error;

/// a substitute for slice type, used in struct members.
/// the purpose is to avoid copy.
/// slice is difficult to use since it takes a lifetime parameter
#[derive(Debug)]
pub struct ByteSequence(pub *const u8, pub usize);

impl Default for ByteSequence {
    fn default() -> Self {
        Self(std::ptr::null(), 0)
    }
}

impl ByteSequence {
	pub fn new(p: *const u8, len: usize) -> Self {
		Self(p, len)
	}

	pub fn GetSlice(&self) -> &[u8] {
		let ptr = self.0;
		let len = self.1;
		unsafe {std::slice::from_raw_parts(ptr, len)}
	}
}


pub fn Read<T: Sized>(data: &[u8]) -> T {
    let sz = size_of::<T>();
    if data.len() < sz {
        error!("failed to read");
    }

    let mut val = unsafe { std::mem::zeroed::<T>() };
    let val_ptr = &mut val as *mut T as *mut u8;
    unsafe {
        std::ptr::copy::<u8>(data.as_ptr(), val_ptr, sz);
    }

    val
}

/// write an element into the buffer named `data`
pub fn Write<T: Sized>(data: &mut [u8], elem: &T) {
    let sz = size_of::<T>();
    if data.len() < sz {
        error!("failed to write. file length = {}, but write size = {sz}", data.len());
    }

    let elem_ptr = std::ptr::addr_of!(*elem) as *const u8;
    let data_ptr = data.as_mut_ptr();

    unsafe {
        std::ptr::copy(elem_ptr, data_ptr, sz);
    }
}

pub fn ReadSlice<T: Sized>(data: &[u8]) -> Vec<T> {
    data.chunks_exact(size_of::<T>())
        .map(|chunk| {
            let ptr = chunk.as_ptr() as *const T;
            unsafe {
                std::ptr::read(ptr)
            }
        } )
        .collect()
}

pub fn atoi(s: &[u8]) -> usize {
	let s = std::str::from_utf8(s).unwrap().trim();
	let end = s.find(" ").unwrap_or(s.len());
	s[0..end].parse::<usize>().unwrap()
}

pub fn AlignTo(val: usize, align: usize) -> usize {
    match align {
        0 => val,
        _ => (val + align - 1) &! (align - 1)
    }
}

pub trait ToRcRefcell {
    fn ToRcRefcell(self) -> Rc<RefCell<Self>>;
}

impl<T> ToRcRefcell for T {
    fn ToRcRefcell(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }
}

/// an ugly function to deal with rust's borrow rules...
//pub fn ptr2ref(ctx_ptr: *mut Box<Context>) -> &'static mut Box<Context> {
//	unsafe{&mut *ctx_ptr}
//}

pub fn ptr2ref<T>(ptr: *mut T) -> &'static mut T {
	unsafe {&mut *ptr}
}