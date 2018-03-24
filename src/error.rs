use std::fmt;

use failure::{Backtrace, Context, Fail};

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    // indicates that the received reply couldn't be decoded (e.g. caused by an aborted
    // connection)
    #[fail(display = "Wrong string format, couldn't parse as UTF8")]
    UTF8Decode,

    // indicates a Telegram error (e.g. a property is missing)
    #[fail(display = "Telegram server responsed with an error")]
    Telegram,

    #[fail(display = "Failed to read file for upload")]
    TelegramFileRead,

    #[fail(display = "There was an error initializing HTTPS")]
    HttpsInitializeError,

    // indicates some failure in Hyper, missing network connection, etc.
    #[fail(display = "There was an error fetching the content")]
    Hyper,

    // indicates some failure with parsing a URI
    #[fail(display = "There was an error parsing the URI")]
    Uri,

    // indicates an error reading or writing data
    #[fail(display = "Failed to read or write data")]
    IO,

    // indicates a malformated reply, this should never happen unless the Telegram server has a
    // hard time
    #[fail(display = "Failed to parse a JSON string")]
    JsonParse,

    #[fail(display = "Failed to serialize to JSON")]
    JsonSerialize,

    #[fail(display = "Json from server is malformed")]
    Json,

    #[fail(display = "Failed to create a channel")]
    Channel,

    #[fail(display = "Failed to create the interval timer")]
    IntervalTimer,

    #[fail(display = "Tokio library caused error")]
    Tokio,

    #[fail(display = "Please specify a file")]
    NoFile,

    #[fail(display = "Expected JSON to be a Map, got something else")]
    JsonNotMap,

    // indicates an unknown error
    #[fail(display = "Unknown error")]
    Unknown,
}

#[derive(Debug, Fail)]
#[fail(display = "{}", message)]
pub struct TelegramError {
    message: String,
}

impl TelegramError {
    pub fn new(message: String) -> Self {
        TelegramError { message }
    }
}
