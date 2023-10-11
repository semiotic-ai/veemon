use crate::dbin::error::DbinFileError;
use crate::headers::error::BlockHeaderError;
use crate::receipts::error::ReceiptError;
use crate::transactions::error::TransactionError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("Invalid input")]
    InvalidInput,
    #[error("Dbin File Error: {0}")]
    DbinFileError(#[from] DbinFileError),
    #[error("Invalid Block Header: {0}")]
    BlockHeaderError(#[from] BlockHeaderError),
    #[error("Invalid Transaction Root: {0}")]
    TransactionRoot(#[from] TransactionError),
    #[error("Invalid Receipt Root: {0}")]
    ReceiptRoot(#[from] ReceiptError),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid content type: {0}")]
    InvalidContentType(String),
    #[error("Invalid protobuf: {0}")]
    ProtobufError(String),
}
