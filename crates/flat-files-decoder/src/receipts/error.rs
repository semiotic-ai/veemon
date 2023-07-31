use thiserror::Error;

#[derive(Error, Debug)]
pub enum InvalidReceiptError {
    #[error("Invalid status")]
    Status,
    #[error("Invalid tx type")]
    TxType,
    #[error("Receipt root mismatch: {0} != {1}")]
    ReceiptRoot(String, String),
    #[error("Invalid address: {0}")]
    Address(String),
    #[error("Invalid topic: {0}")]
    Topic(String),
}