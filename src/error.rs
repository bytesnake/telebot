use std::fmt;

use failure::{Backtrace, Context, Fail};
use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    // indicates that the received reply couldn't be decoded (e.g. caused by an aborted
    // connection)
    #[fail(display = "Wrong string format, couldn't parse as UTF8")]
    UTF8Decode,
    // indicates a Telegram error (e.g. a property is missing)
    #[fail(display = "Telegram server responsed with an error")]
    Telegram,
    // indicates some failure in Tokio-CURL, missing network connection etc.
    #[fail(display = "Failure in the Tokio-cURL library")]
    TokioCurl,
    // indicates some failure in CURL, failed to configure request, etc.
    #[fail(display = "Failed to configure request")]
    cURL,
    // indicates some failure in Curl's Form module, failed to build request, etc.
    #[fail(display = "Failed to build form part of the request")]
    Form,
    // indicates an error reading or writing data
    #[fail(display = "Failed to read or write data")]
    IO,
    // indicates a malformated reply, this should never happen unless the Telegram server has a
    // hard time
    #[fail(display = "Failed to parse a JSON string")]
    JSON,

    #[fail(display = "Failed to create a channel")]
    Channel,

    #[fail(display = "Failed to create the interval timer")]
    IntervalTimer,

    #[fail(display = "Tokio library caused error")]
    Tokio,

    #[fail(display = "Please specify a file")]
    NoFile,

    // indicates an unknown error
    #[fail(display = "Unknown error")]
    Unknown,
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner: inner }
    }
}
