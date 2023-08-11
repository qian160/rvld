const MAGIC: &[u8] = "\x7fELF".as_bytes();

pub fn checkMagic(s: &Vec<u8>) -> bool {
    s.starts_with(MAGIC)
}