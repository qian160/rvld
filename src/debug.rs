use std::fmt;
use std::io::Write;

use crate::linker::elf::{Sym, Shdr, Ehdr};

/// `31` = red, `32` = green, `33` = yellow
#[macro_export]
macro_rules! color_text {
    ($text:expr, $color:expr) => {{
        format!("\x1b[{}m{}\x1b[0m", $color, $text)
    }};
}

pub fn print(args: fmt::Arguments) {
    std::io::stdout().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! error{
    ($fmt: literal $(, $($arg: tt)+)?) => {
        print(format_args!(concat!("\x1b[0;1;91m", $fmt, "\n\x1b[0m") $(, $($arg)+)?));
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


impl std::fmt::Debug for Ehdr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "
Ident:		{:x?}
Type:		{:?}
Machine:	{:?}
Version:	{:?}
Entry:		{:?}
PhOff:		{:?}
ShOff:		{:?}
Flags:		{:?}
EhSize:		{:?}
PhEntSize:	{:?}
PhNum:		{:?}
ShEntSize:	{:?}
ShNum:		{:?}
ShStrndx:	{:?}
		", self.Ident, self.Type, self.Machine, self.Version, self.Entry, self.PhOff, self.ShOff, self.Flags, self.EhSize, self.PhEntSize, self.PhNum, self.ShEntSize, self.ShNum, self.ShStrndx)
	}
}

impl std::fmt::Debug for Shdr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "
Name:		{}
Type:		{}
Flags:		{}
Addr:		{}
Offset:		{}
Size:		{}
Link:		{}
Info:		{}
AddrAlign:	{}
EntSize:	{}
		", self.Name, self.Type, self.Flags, self.Addr, self.Offset, self.Size, self.Link, self.Info, self.AddrAlign, self.EntSize)
	}
}

impl std::fmt::Debug for Sym {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "
Name:		{:x?}
Info:		{:?}
Other:		{:?}
Shndx:		{:?}
Val:		{:?}
Size:		{:?}
		", self.Name, self.Info, self.Other, self.Shndx, self.Val, self.Size)
	}
}