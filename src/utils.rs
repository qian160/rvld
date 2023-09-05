use std::mem::size_of;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::{Add, Sub, Deref};
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

#[derive(Debug)]
pub struct CheckedU64(pub u64);

impl Add for CheckedU64 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::from(self.0.wrapping_add(rhs.0))
    }
}

impl Sub for CheckedU64{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::from(self.0.wrapping_sub(rhs.0))
    }
}

impl From<u64> for CheckedU64 {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl Deref for CheckedU64 {
    type Target = u64;
    fn deref(&self) -> &u64 {
        &self.0
    }
}
/// an example usage:
/// 
/// let foo = Rc::new(RefCell::new(114514));
/// 
/// let p = Ptr::new(ptr2ref(foo.as_ptr()));
/// 
/// debug!("{}", p.is_null());  // false
/// 
/// debug!("{}", p.get());      // 114514
/// 
/// *p.get() = 1;
/// 
/// debug!("{}", a.get());      // 1
/*
pub struct Ptr<T> {
    _Contents:   *mut T,
}

impl <T> Default for Ptr<T> {
    fn default() -> Self {
        Ptr { _Contents: std::ptr::null_mut() }
    }
}

impl <T> Ptr<T> {
    /// note: can not point to local variables
    pub fn new(c: &mut T) -> Self {
        Self { _Contents: c }
    }
    pub fn is_null(&self) -> bool {
        self._Contents.is_null()
    }
    pub fn get(&self) -> &mut T{
        unsafe { &mut *self._Contents }
    }
}
 */
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
pub fn Write<T: Sized>(loc: &mut [u8], elem: T) {
    let sz = size_of::<T>();
    if loc.len() < sz {
        error!("failed to write. file length = {}, but write size = {sz}", loc.len());
    }

    let elem_ptr = std::ptr::addr_of!(elem) as *const u8;
    let loc_ptr = loc.as_mut_ptr();

    unsafe {
        std::ptr::copy(elem_ptr, loc_ptr, sz);
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

pub fn vec2slice<T>(v: &Vec<T>) -> &[u8] {
    let len = v.len() * std::mem::size_of::<T>();
    let ptr = v.as_ptr() as *const u8;

    unsafe {std::slice::from_raw_parts(ptr, len)}
}

pub fn AlignTo(val: usize, align: usize) -> usize {
    match align {
        0 => val,
        _ => (val + align - 1) & !(align - 1)
    }
}

#[allow(unused)]
pub fn hasSingleBit(n: u64) -> bool {
    n & (n - 1) == 0
}

#[allow(unused)]
pub fn BitCeil(val: u64) -> u64 {
    if hasSingleBit(val) {
        return val;
    }

    1 << (64 - val.leading_zeros() as u64 )
}

pub fn Bit<T: Into<u32> + Copy>(val: T, pos: u32) -> T {
    let val: u32 = val.into();
    let res = (val >> pos) & 1;
    unsafe {*(&res as *const u32 as *const T)}
}

//pub fn Bits(val: u32, hi: u32, lo: u32) -> u32 {
pub fn Bits<T: Into<u32> + Copy>(val: T, hi: u32, lo: u32) -> T {
    let val: u32 = val.into();
    let res = (val >> lo) & ((1 << (hi - lo + 1)) - 1);
    unsafe {*(&res as *const u32 as *const T)}
}

// mark
#[allow(unused)]
pub fn SignExtend(val: u64, size: i32) -> u64 {
    ((val<<(63-size)) as i64 >> (63 - size) as i64) as u64
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

pub fn default<T: Default>() -> T {
    T::default()
}