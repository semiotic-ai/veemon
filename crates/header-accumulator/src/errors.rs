use alloy_primitives::map::HashSet;
use firehose_protos::error::ProtosError;

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

    #[error("blocks in epoch must be exactly 8192 units, found {0}")]
    InvalidEpochLength(usize),

    #[error("block was missing while creating epoch")]
    MissingBlock(Vec<u64>),

    #[error("not all blocks are in the same epoch. epochs found: {0:?}")]
    InvalidBlockInEpoch(HashSet<u64>),
    #[error("Error converting ExtHeaderRecord to header")]
    ExtHeaderRecordError,
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
