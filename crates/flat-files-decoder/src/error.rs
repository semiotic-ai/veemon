use crate::dbin::DbinFileError;
use crate::headers::BlockHeaderError;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("Invalid input")]
    InvalidInput,
    #[error("error decompressing")]
    DecompressError,
    #[error("Dbin File Error: {0}")]
    DbinFileError(#[from] DbinFileError),
    #[error("Invalid Block Header: {0}")]
    BlockHeaderError(#[from] BlockHeaderError),
    #[error("Invalid Transaction Root")]
    TransactionRoot,
    #[error("Invalid Receipt Root")]
    ReceiptRoot,
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid content type: {0}")]
    InvalidContentType(String),
    #[error("Invalid protobuf: {0}")]
    ProtobufError(String),
    #[error("Join error: {0}")]
    JoinError(JoinError),
}
