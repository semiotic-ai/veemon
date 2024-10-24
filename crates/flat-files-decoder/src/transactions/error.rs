use firehose_protos::error::ProtosError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Mismatched Transaction Root: {0} != {1}")]
    MismatchedRoot(String, String),
    #[error("Missing call field")]
    MissingCall,
    #[error("Invalid BigInt")]
    InvalidBigInt(String),
    #[error("EIP-4844 not supported")]
    EIP4844NotSupported,
    #[error("Missing Gas Price")]
    MissingGasPrice,
    #[error("Missing Value")]
    MissingValue,
    #[error("Missing Max Fee Per Gas")]
    MissingMaxFeePerGas,
    #[error("Missing Header")]
    MissingHeader,
    #[error("{0}")]
    ProtosError(#[from] ProtosError),
}
