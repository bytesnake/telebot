use std::io;
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
    // indicates an unknown error
    Unknown,
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
