#[derive(Debug)]
pub enum Error {
    UTF8Decode,
    Telegram(String),
    TokioCurl,
    JSON,
    Unknown
}

