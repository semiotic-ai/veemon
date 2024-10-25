use firehose_protos::error::ProtosError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReceiptError {
    #[error("Invalid status")]
    InvalidStatus,
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Invalid topic: {0}")]
    InvalidTopic(String),
    #[error("Invalid data: {0}")]
    InvalidBloom(String),
    #[error("Receipt root mismatch: {0} != {1}")]
    MismatchedRoot(String, String),
    #[error("Missing receipt root")]
    MissingRoot,
    #[error("Missing receipt")]
    MissingReceipt,
    #[error("Protos error: {0}")]
    ProtosError(#[from] ProtosError),
    #[error("TryFromSliceError: {0}")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
}
