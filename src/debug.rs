use std::fmt;
use std::io::Write;

use crate::linker::elf::{Sym, Shdr, Ehdr, ArHdr};

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
        crate::debug::print(format_args!(concat!("\x1b[0;1;91m", $fmt, "\n\x1b[0m") $(, $($arg)+)?));
        std::process::exit(1);
    };
}

#[macro_export]
macro_rules! warn{
    ($fmt: literal $(, $($arg: tt)+)?) => {
        crate::debug::print(format_args!(concat!("\x1b[93m", $fmt, "\n\x1b[0m") $(, $($arg)+)?));
    };
}

#[macro_export]
macro_rules! info{
    ($fmt: literal $(, $($arg: tt)+)?) => {
        crate::debug::print(format_args!(concat!("\x1b[94m", $fmt, "\n\x1b[0m") $(, $($arg)+)?));
    };
}

#[macro_export]
macro_rules! debug{
    ($fmt: literal $(, $($arg: tt)+)?) => {
        crate::debug::print(format_args!(concat!("\x1b[92m", $fmt, "\n\x1b[0m") $(, $($arg)+)?));
    };
}


impl std::fmt::Debug for Ehdr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "\
            Ident:		{:x?}\n\
            Type:		{:?}\n\
            Machine:	{:?}\n\
            Version:	{:?}\n\
            Entry:		{:?}\n\
            PhOff:		{:?}\n\
            ShOff:		{:?}\n\
            Flags:		{:?}\n\
            EhSize:		{:?}\n\
            PhEntSize:	{:?}\n\
            PhNum:		{:?}\n\
            ShEntSize:	{:?}\n\
            ShNum:		{:?}\n\
            ShStrndx:	{:?}\n\
		", self.Ident, self.Type, self.Machine, self.Version, self.Entry, self.PhOff, self.ShOff, self.Flags, self.EhSize, self.PhEntSize, self.PhNum, self.ShEntSize, self.ShNum, self.ShStrndx)
	}
}

impl std::fmt::Debug for Shdr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "\
            Name:		{}\n\
            Type:		{}\n\
            Flags:		{}\n\
            Addr:		{}\n\
            Offset:		{}\n\
            Size:		{}\n\
            Link:		{}\n\
            Info:		{}\n\
            AddrAlign:	{}\n\
            EntSize:	{}\n\
		", self.Name, self.Type, self.Flags, self.Addr, self.Offset, self.Size, self.Link, self.Info, self.AddrAlign, self.EntSize)
	}
}

impl std::fmt::Debug for Sym {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "\
            Name:		{:x?}\n\
            Info:		{:?}\n\
            Other:		{:?}\n\
            Shndx:		{:?}\n\
            Val:		{:?}\n\
            Size:		{:?}\n\
		", self.Name, self.Info, self.Other, self.Shndx, self.Val, self.Size)
	}
}

impl std::fmt::Debug for ArHdr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "\
            Name:	\"{}\"  {:x?}\n\
            Data:	\"{}\"\n\
            Uid:	\"{}\"\n\
            Gid:	\"{}\"\n\
            Mode:	\"{}\"\n\
            Size:	\"{}\"\n\
            Fmag:	\"{:x?}\"\n\
		", std::str::from_utf8(&self.Name.to_vec()).unwrap(), &self.Name,
        std::str::from_utf8(&self.Date.to_vec()).unwrap(),
        std::str::from_utf8(&self.Uid.to_vec()).unwrap(),
        std::str::from_utf8(&self.Gid.to_vec()).unwrap(),
        std::str::from_utf8(&self.Mode.to_vec()).unwrap(),
        std::str::from_utf8(&self.Size.to_vec()).unwrap(),
        self.Fmag)
	}
}