use crate::headers::BlockHeaderRoots;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockHeaderError {
    #[error("Read error: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("JSON Error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Mismatched roots: {0:?}")]
    MismatchedRoots(Box<(BlockHeaderRoots, BlockHeaderRoots)>),
    #[error("Missing header")]
    MissingHeader,
    #[error("Invalid total difficulty")]
    InvalidTotalDifficulty,
}
