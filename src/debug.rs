use crate::linker::elf::{Sym, Shdr, Ehdr};
use crate::linker::archive::ArHdr;

/// `31` = red, `32` = green, `33` = yellow, 34 = blue
#[macro_export]
macro_rules! color_text {
    ($text:expr, $color:expr) => {{
        format!("\x1b[{}m{}\x1b[0m", $color, $text)
    }};
}

// green
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        println!("\x1b[92m[{} - {}]{}\x1b[0m", file!(), line!(), format_args!($($arg)*));
    };
}

/// blue
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!("\x1b[94m[{} - {}]{}\x1b[0m", file!(), line!(), format_args!($($arg)*));
    };
}

/// yellow
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        println!("\x1b[93m[{} - {}]{}\x1b[0m", file!(), line!(), format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        println!("\x1b[0;1;91m[{} - {}]{}\x1b[0m", file!(), line!(), format_args!($($arg)*));
        std::process::exit(114514);
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
		", 
        std::str::from_utf8(&self.Name.to_vec()).unwrap(), &self.Name,
        std::str::from_utf8(&self.Date.to_vec()).unwrap(),
        std::str::from_utf8(&self.Uid.to_vec()).unwrap(),
        std::str::from_utf8(&self.Gid.to_vec()).unwrap(),
        std::str::from_utf8(&self.Mode.to_vec()).unwrap(),
        std::str::from_utf8(&self.Size.to_vec()).unwrap(),
        self.Fmag)
	}
}