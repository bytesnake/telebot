use std::io::Read;
use std::fs;

pub struct File {
    pub name: String,
    pub source: Box<Read>
}

impl<'a> From<&'a str> for File {
    fn from(path: &'a str) -> File {
        let file = fs::File::open(path).unwrap();

        File { name: path.into(), source: Box::new(file) }
    }
}

impl<'a, S: Read+'static> From<(&'a str, S)> for File {
    fn from((path, source): (&'a str, S)) -> File where S: Read + 'static {
        File { name: path.into(), source: Box::new(source) }
    }
}

