use std::io;
use std::fmt;
use std::str;
use std::sync::PoisonError;
use std::error::Error as StdError;
use serde_json::Error as JsonError;
use hyper::Error as HyperError;
use hyper::error::UriError;
use native_tls::Error as TlsError;

#[derive(Debug)]
pub enum Error {
    // indicates that the received reply couldn't be decoded (e.g. caused by an aborted
    // connection)
    UTF8Decode,
    // indicates a Telegram error (e.g. a property is missing)
    Telegram(String),
    // indicates some failure in Hyper, missing network connection, etc.
    Hyper(HyperError),
    // indicates some failure with parsing a URI
    Uri(UriError),
    // indicates some failure with HTTPS
    Tls(TlsError),
    // indicates an error reading or writing data
    IO(io::Error),
    // indicates a malformated reply, this should never happen unless the Telegram server has a
    // hard time
    JSON,
    // indicates whether a file was supposed to be attached, but wasn't properly read
    NoFile,
    // indicates an unknown error
    Unknown,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())?;

        if let &Error::Telegram(ref message) = self {
            write!(f, ": {}", message)?;
        }

        Ok(())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            &Error::UTF8Decode => "reply could not be decoded",
            &Error::Telegram(_) => "telegram returned error",
            &Error::Hyper(_) => "hyper error",
            &Error::Uri(_) => "uri error",
            &Error::Tls(_) => "tls error",
            &Error::IO(_) => "error reading or writing data",
            &Error::JSON => "malformed reply",
            &Error::NoFile => "error reading attached file",
            &Error::Unknown => "unknown error",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match self {
            &Error::UTF8Decode |
            &Error::JSON |
            &Error::NoFile |
            &Error::Unknown => None,
            &Error::Telegram(_) => None,
            &Error::Hyper(_) => None,
            &Error::Uri(_) => None,
            &Error::Tls(_) => None,
            &Error::IO(ref e) => Some(e),
        }
    }
}

impl From<JsonError> for Error {
    fn from(_: JsonError) -> Self {
        Error::JSON
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Error::Unknown
    }
}

impl From<str::Utf8Error> for Error {
    fn from(_: str::Utf8Error) -> Self {
        Error::UTF8Decode
    }
}

impl From<HyperError> for Error {
    fn from(err: HyperError) -> Self {
        Error::Hyper(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}

impl From<UriError> for Error {
    fn from(err: UriError) -> Self {
        Error::Uri(err)
    }
}

impl From<TlsError> for Error {
    fn from(err: TlsError) -> Self {
        Error::Tls(err)
    }
}
