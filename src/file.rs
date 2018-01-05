//! A Telegram file which contains a readable source and a filename
//!
//! The filename should be such that it represents the content type.

use std::io::Read;
use std::fs;
use std::convert::TryFrom;
use failure::{Error, ResultExt};
use error::ErrorKind;

/// A Telegram file which contains a readable source and a filename
pub struct File {
    pub name: String,
    pub source: Box<Read>,
}

/// Construct a Telegram file from a local path
impl<'a> TryFrom<&'a str> for File {
    type Error = Error;

    fn try_from(path: &'a str) -> Result<Self, Error> {
        let file = fs::File::open(path).context(ErrorKind::IO)?;

        Ok(File {
            name: path.into(),
            source: Box::new(file),
        })
    }
}

/// Construct a Telegram file from an object which implements the Read trait
impl<'a, S: Read + 'static> TryFrom<(&'a str, S)> for File {
    type Error = Error;

    fn try_from((path, source): (&'a str, S)) -> Result<Self, Error>
    where
        S: Read + 'static,
    {
        Ok(File {
            name: path.into(),
            source: Box::new(source),
        })
    }
}
