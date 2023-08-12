use std::fmt;
use std::io::Write;

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
