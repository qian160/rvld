/// `31` = red, `32` = green, `33` = yellow
use std::fmt;
use std::io::Write;
use std::mem::size_of;

#[macro_export]
macro_rules! color_text {
    ($text:expr, $color:expr) => {{
        format!("\x1b[{}m{}\x1b[0m", $color, $text)
    }};
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        print(format_args!($fmt $(, $($arg)+)?));
    }
}

pub fn print(args: fmt::Arguments) {
    std::io::stdout().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! error{
    ($fmt: literal $(, $($arg: tt)+)?) => {
        print(format_args!(concat!("\x1b[91m", $fmt, "\n\x1b[0m") $(, $($arg)+)?));
        std::process::exit(1);
    };
}

#[macro_export]
macro_rules! warn{
    ($fmt: literal $(, $($arg: tt)+)?) => {
        print(format_args!(concat!("\x1b[93m", $fmt, "\n\x1b[0m") $(, $($arg)+)?));
    };
}

#[macro_export]
macro_rules! info{
    ($fmt: literal $(, $($arg: tt)+)?) => {
        print(format_args!(concat!("\x1b[94m", $fmt, "\n\x1b[0m") $(, $($arg)+)?));
    };
}

#[macro_export]
macro_rules! debug{
    ($fmt: literal $(, $($arg: tt)+)?) => {
        print(format_args!(concat!("\x1b[92m", $fmt, "\n\x1b[0m") $(, $($arg)+)?));
    };
}

pub fn Read<T: Sized>(data: &[u8]) -> Option<T> {
    let sz = size_of::<T>();
    if data.len() < sz {
        return None;
    }

    let mut val = unsafe{ std::mem::zeroed::<T>()};

    let value_ptr = &mut val as *mut T as *mut u8;

    unsafe {
        std::ptr::copy_nonoverlapping(data.as_ptr(), value_ptr, sz);
    }

    Some(val)
}