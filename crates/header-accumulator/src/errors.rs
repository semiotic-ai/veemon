use std::fmt;

use firehose_protos::error::ProtosError;

#[derive(thiserror::Error, Debug)]
pub enum EraValidateError {
    #[error("Too many header records")]
    TooManyHeaderRecords,
    #[error("Invalid pre-merge accumulator file")]
    InvalidPreMergeAccumulatorFile,
    #[error("Error decoding header from flat files")]
    HeaderDecodeError,
    #[error("Error decoding flat files")]
    FlatFileDecodeError,
    #[error("Era accumulator mismatch")]
    EraAccumulatorMismatch,
    #[error("Error creating epoch accumulator")]
    EpochAccumulatorError,
    #[error("Error generating inclusion proof")]
    ProofGenerationFailure,
    #[error("Error validating inclusion proof")]
    ProofValidationFailure,
    #[error("Error reading from stdin")]
    IoError,
    #[error("Start epoch block not found")]
    StartEpochBlockNotFound,
    #[error("Start epoch must be less than end epoch")]
    EndEpochLessThanStartEpoch,
    #[error("Merge block not found")]
    MergeBlockNotFound,
    #[error("Error reading json from stdin")]
    JsonError,
    #[error("Error decoding total difficulty")]
    TotalDifficultyDecodeError,
    #[error("blocks in epoch must respect the range of blocks numbers")]
    InvalidEpochStart,
    #[error("blocks in epoch must be exactly 8192 units, found {0}")]
    InvalidEpochLength(usize),

    #[error("not all blocks are in the same epoch")]
    InvalidBlockInEpoch,
    #[error("Error converting ExtHeaderRecord to header")]
    ExtHeaderRecordError,
    #[error("Invalid block range: {0} - {1}")]
    InvalidBlockRange(u64, u64),
}

impl From<ProtosError> for EraValidateError {
    fn from(error: ProtosError) -> Self {
        match error {
            ProtosError::BlockConversionError => Self::HeaderDecodeError,
            _ => unimplemented!("Error mapping is not implemented"),
        }
    }
}
