use std::mem::size_of;
use crate::warn;
pub fn Read<T: Sized>(data: &[u8]) -> Option<T> {
    let sz = size_of::<T>();

    if data.len() < sz {
        warn!("read failed. actual size = {}", data.len());
        return None;
    }
    let mut val = unsafe{ std::mem::zeroed::<T>()};
    let val_ptr = &mut val as *mut T as *mut u8;
    unsafe {
        std::ptr::copy::<u8>(data.as_ptr(), val_ptr, sz);
    }

    Some(val)
}