//! A Telegram file which contains a readable source and a filename
//!
//! The filename should be such that it represents the content type.

use std::{io::Read, convert::TryFrom, path::Path};
use failure::Error;
use error::ErrorKind;

/// A Telegram file which contains a readable source and a filename
pub enum File {
    Memory {
        name: String,
        source: Box<Read + Send>,
    },
    Disk {
        path: String,
    },
}

/// Construct a Telegram file from a local path
impl<'a> TryFrom<&'a str> for File {
    type Error = Error;

    fn try_from(path: &'a str) -> Result<Self, Self::Error> {
        let file = Path::new(path);

        if file.is_file() {
            Ok(File::Disk { path: path.into() })
        } else {
            Err(Error::from(ErrorKind::NoFile))
        }
    }
}

/// Construct a Telegram file from an object which implements the Read trait
impl<'a, S: Read + Send + 'static> TryFrom<(&'a str, S)> for File {
    type Error = Error;

    fn try_from((name, source): (&'a str, S)) -> Result<Self, Self::Error>
    where
        S: Read + Send,
    {
        Ok(File::Memory {
            name: name.into(),
            source: Box::new(source),
        })
    }
}
