use std::fmt;

#[derive(Debug)]
pub enum EraValidateError {
    TooManyHeaderRecords,
    InvalidPreMergeAccumulatorFile,
    HeaderDecodeError,
    FlatFileDecodeError,
    EraAccumulatorMismatch,
    EpochAccumulatorError,
    ProofGenerationFailure,
    ProofValidationFailure,
    IoError,
    StartEpochBlockNotFound,
    EndEpochLessThanStartEpoch,
    MergeBlockNotFound,
    JsonError,
    TotalDifficultyDecodeError,
    InvalidEpochStart,
    InvalidEpochLength,
}
#[derive(Debug)]
pub enum SyncError {
    LockfileReadError,
}

impl std::error::Error for EraValidateError {}
impl std::error::Error for SyncError {}

impl fmt::Display for EraValidateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EraValidateError::TooManyHeaderRecords => write!(f, "Too many header records"),
            EraValidateError::InvalidPreMergeAccumulatorFile => {
                write!(f, "Invalid master accumulator file")
            }
            EraValidateError::HeaderDecodeError => {
                write!(f, "Error decoding header from flat files")
            }
            EraValidateError::FlatFileDecodeError => write!(f, "Error decoding flat files"),
            EraValidateError::EraAccumulatorMismatch => write!(f, "Era accumulator mismatch"),
            EraValidateError::EpochAccumulatorError => {
                write!(f, "Error creating epoch accumulator")
            }
            EraValidateError::ProofGenerationFailure => {
                write!(f, "Error generating inclusion proof")
            }
            EraValidateError::ProofValidationFailure => {
                write!(f, "Error validating inclusion proof")
            }
            EraValidateError::IoError => write!(f, "Error reading from stdin"),
            EraValidateError::StartEpochBlockNotFound => {
                write!(f, "Start epoch block not found")
            }
            EraValidateError::EndEpochLessThanStartEpoch => {
                write!(f, "Start epoch must be less than end epoch")
            }
            EraValidateError::MergeBlockNotFound => {
                write!(f, "Merge block not found")
            }
            EraValidateError::JsonError => {
                write!(f, "Error reading json from stdin")
            }
            EraValidateError::TotalDifficultyDecodeError => {
                write!(f, "Error decoding total difficulty")
            }
            EraValidateError::InvalidEpochLength => {
                write!(f, "blocks in epoch must be exactly 8192 units")
            }
            EraValidateError::InvalidEpochStart => {
                write!(
                    f,
                    "blocks in epoch must respect the range of blocks numbers"
                )
            }
        }
    }
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error handling lockfile sync")
    }
}
