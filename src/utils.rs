use std::mem::size_of;
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