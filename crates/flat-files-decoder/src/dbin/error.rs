use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbinFileError {
    #[error("Incorrect dbin bytes")]
    InvalidDBINBytes,
    #[error("Read error")]
    ReadError(#[from] std::io::Error),
    #[error("Invalid UTF8")]
    InvalidUTF8(#[from] std::string::FromUtf8Error),
}