use std::{mem::size_of, rc::Rc, cell::RefCell};
use crate::error;

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
pub fn Write<T: Sized>(data: &mut [u8], elem: T) {
    let sz = size_of::<T>();
    if data.len() < sz {
        error!("failed to write. file length = {}, but write size = {sz}", data.len());
    }

    let elem_ptr = std::ptr::addr_of!(elem) as *const u8;
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