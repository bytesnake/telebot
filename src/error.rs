use std::io;
use std::fmt;
use std::str;
use std::sync::PoisonError;
use std::error::Error as StdError;
use serde_json::Error as JsonError;
use curl::Error as CurlError;
use curl::FormError;
use tokio_curl::PerformError;

#[derive(Debug)]
pub enum Error {
    // indicates that the received reply couldn't be decoded (e.g. caused by an aborted
    // connection)
    UTF8Decode,
    // indicates a Telegram error (e.g. a property is missing)
    Telegram(String),
    // indicates some failure in Tokio-CURL, missing network connection etc.
    TokioCurl(PerformError),
    // indicates some failure in CURL, failed to configure request, etc.
    Curl(CurlError),
    // indicates some failure in Curl's Form module, failed to build request, etc.
    Form(FormError),
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
            &Error::TokioCurl(_) => "tokio-curl error",
            &Error::Curl(_) => "curl error",
            &Error::Form(_) => "curl form error",
            &Error::IO(_) => "error reading or writing data",
            &Error::JSON => "malformed reply",
            &Error::NoFile => "error reading attached file",
            &Error::Unknown => "unknown error",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match self {
            &Error::UTF8Decode | &Error::JSON | &Error::NoFile | &Error::Unknown => None,
            &Error::Telegram(_) => None,
            &Error::TokioCurl(ref e) => Some(e),
            &Error::Curl(ref e) => Some(e),
            &Error::Form(ref e) => Some(e),
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

impl From<PerformError> for Error {
    fn from(err: PerformError) -> Self {
        Error::TokioCurl(err)
    }
}

impl From<CurlError> for Error {
    fn from(err: CurlError) -> Self {
        Error::Curl(err)
    }
}

impl From<FormError> for Error {
    fn from(err: FormError) -> Self {
        Error::Form(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}
