//! A Telegram file which contains a readable source and a filename
//!
//! The filename should be such that it represents the content type.

use std::io::{self, Read};
use std::fs;
use std::convert::TryFrom;

/// A Telegram file which contains a readable source and a filename
pub struct File {
    pub name: String,
    pub source: Box<Read>,
}

/// Construct a Telegram file from a local path
impl<'a> TryFrom<&'a str> for File {
    type Error = io::Error;

    fn try_from(path: &'a str) -> Result<Self, Self::Error> {
        let file = fs::File::open(path)?;

        Ok(File {
            name: path.into(),
            source: Box::new(file),
        })
    }
}

/// Construct a Telegram file from an object which implements the Read trait
impl<'a, S: Read + 'static> From<(&'a str, S)> for File {
    fn from((path, source): (&'a str, S)) -> File
    where
        S: Read + 'static,
    {
        File {
            name: path.into(),
            source: Box::new(source),
        }
    }
}
