use alloy_primitives::map::HashSet;
use firehose_protos::ProtosError;

/// Possible errors while interacting with the lib
#[derive(thiserror::Error, Debug)]
pub enum EraValidateError {
    #[error("Error decoding header from flat files")]
    HeaderDecodeError,
    #[error("Era accumulator mismatch")]
    EraAccumulatorMismatch,

    #[error("Error generating inclusion proof")]
    ProofGenerationFailure,
    #[error("Error validating inclusion proof")]
    ProofValidationFailure,

    #[error("Blocks in epoch must be exactly 8192 units, found {0}")]
    InvalidEpochLength(usize),

    #[error("Block was missing while creating epoch {epoch}. Missing blocks: {blocks:?}")]
    MissingBlock { epoch: u64, blocks: Vec<u64> },

    #[error("Not all blocks are in the same epoch. Epochs found: {0:?}")]
    InvalidBlockInEpoch(HashSet<u64>),
    #[error("Error converting ExtHeaderRecord to header block number {0}")]
    ExtHeaderRecordError(u64),
    #[error("Invalid block range: {0} - {1}")]
    InvalidBlockRange(u64, u64),
    #[error("Epoch is in post merge: {0}")]
    EpochPostMerge(usize),
}

impl From<ProtosError> for EraValidateError {
    fn from(error: ProtosError) -> Self {
        match error {
            ProtosError::BlockConversionError => Self::HeaderDecodeError,
            _ => unimplemented!("Error mapping is not implemented"),
        }
    }
}
