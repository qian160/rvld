#![feature(pattern)]
mod utils;
mod linker;
use linker::elf::*;

use linker::checkMagic;
use std::{env, fs::{File, self}};

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() >= 2, "need an argument");

    let file = &args[1];
    let contents = fs::read(file)
        .expect(&format!("{}: not found!", file));

    assert!(checkMagic(&contents));
    println!("{}", color_text!("ok", 32));
}
