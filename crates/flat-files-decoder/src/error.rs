use crate::headers::BlockHeaderError;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("Invalid Block Header: {0}")]
    BlockHeaderError(#[from] BlockHeaderError),
    #[error("error decompressing")]
    DecompressError,
    #[error("dbin files with different versions")]
    DifferingDbinVersions,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid content type: {0}")]
    InvalidContentType(String),
    #[error("Incorrect dbin bytes")]
    InvalidDbinBytes,
    #[error("Invalid input")]
    InvalidInput,
    #[error("Invalid UTF8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("Join error: {0}")]
    JoinError(JoinError),
    #[error("Invalid protobuf: {0}")]
    ProtobufError(String),
    #[error("Invalid Receipt Root")]
    ReceiptRoot,
    #[error("Start of new dbin file")]
    StartOfNewDbinFile,
    #[error("Invalid Transaction Root")]
    TransactionRoot,
    #[error("Unsupported version")]
    UnsupportedDbinVersion,
}
