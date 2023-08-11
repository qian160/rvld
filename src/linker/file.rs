pub struct File {
    pub Name:       String,
    pub Contents:   Vec<u8>,
}

pub fn newFile(name: &str) -> Box<File> {
    let Contents = std::fs::read(name).expect(&format!("{}: open failed", name));
    Box::new(File{Name: name.to_string(), Contents})
}