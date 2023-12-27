use crate::headers::BlockHeaderRoots;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockHeaderError {
    #[error("Read error")]
    ReadError(#[from] std::io::Error),
    #[error("JSON Error")]
    JsonError(#[from] serde_json::Error),
    #[error("Invalid input")]
    InvalidInput,
    #[error("Mismatched roots: expected {0:?}, got {1:?}")]
    MismatchedRoots(BlockHeaderRoots, BlockHeaderRoots),
    #[error("Missing header")]
    MissingHeader,
    #[error("Invalid total difficulty")]
    InvalidTotalDifficulty,
}
