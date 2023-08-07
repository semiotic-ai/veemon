use crate::transactions::signature::InvalidSignatureError;
use crate::transactions::tx_type::TransactionTypeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Mismatched Transaction Root: {0} != {1}")]
    MismatchedRoot(String, String),
    #[error("Missing call field")]
    MissingCall,
    #[error("Invalid Storage Key: {0}")]
    InvalidStorageKey(String),
    #[error("Invalid BigInt: {0}")]
    InvalidBigInt(String),
    #[error("EIP-4844 not supported")]
    EIP4844NotSupported,
    #[error("Invalid Signature: {0}")]
    InvalidSignature(#[from] InvalidSignatureError),
    #[error("Invalid Transaction Type: {0}")]
    InvalidType(#[from] TransactionTypeError),
}
