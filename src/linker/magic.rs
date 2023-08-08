pub fn checkMagic(s: &Vec<u8>) -> bool {
    let magic = "\x7fELF".as_bytes();
    s.starts_with(magic)
}