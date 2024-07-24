use std::fmt;

use sf_protos::StreamingFastProtosError;

#[derive(Debug)]
pub enum HeaderAccumulatorError {
    EraValidateError(EraValidateError),
    SyncError(SyncError),
}

impl std::error::Error for HeaderAccumulatorError {}

impl fmt::Display for HeaderAccumulatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HeaderAccumulatorError::EraValidateError(ref e) => write!(f, "{}", e),
            HeaderAccumulatorError::SyncError(ref e) => write!(f, "{}", e),
        }
    }
}

impl From<EraValidateError> for HeaderAccumulatorError {
    fn from(error: EraValidateError) -> Self {
        HeaderAccumulatorError::EraValidateError(error)
    }
}

impl From<SyncError> for HeaderAccumulatorError {
    fn from(error: SyncError) -> Self {
        HeaderAccumulatorError::SyncError(error)
    }
}

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
    ExtHeaderRecordError,
    InvalidBlockRange(u64, u64),
}

#[derive(Debug)]
pub enum SyncError {
    LockfileIoError(std::io::Error),
    LockfileReadError,
}

impl std::error::Error for EraValidateError {}
impl std::error::Error for SyncError {}

impl fmt::Display for EraValidateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EraValidateError::*;
        match *self {
            EraValidateError::TooManyHeaderRecords => write!(f, "Too many header records"),
            EraValidateError::InvalidPreMergeAccumulatorFile => {
                write!(f, "Invalid pre-merge accumulator file")
            }
            HeaderDecodeError => {
                write!(f, "Error decoding header from flat files")
            }
            FlatFileDecodeError => write!(f, "Error decoding flat files"),
            EraAccumulatorMismatch => write!(f, "Era accumulator mismatch"),
            EpochAccumulatorError => {
                write!(f, "Error creating epoch accumulator")
            }
            ProofGenerationFailure => {
                write!(f, "Error generating inclusion proof")
            }
            ProofValidationFailure => {
                write!(f, "Error validating inclusion proof")
            }
            IoError => write!(f, "Error reading from stdin"),
            StartEpochBlockNotFound => {
                write!(f, "Start epoch block not found")
            }
            EndEpochLessThanStartEpoch => {
                write!(f, "Start epoch must be less than end epoch")
            }
            MergeBlockNotFound => {
                write!(f, "Merge block not found")
            }
            JsonError => {
                write!(f, "Error reading json from stdin")
            }
            TotalDifficultyDecodeError => {
                write!(f, "Error decoding total difficulty")
            }
            InvalidEpochLength => {
                write!(f, "blocks in epoch must be exactly 8192 units")
            }
            InvalidEpochStart => {
                write!(
                    f,
                    "blocks in epoch must respect the range of blocks numbers"
                )
            }
            ExtHeaderRecordError => {
                write!(f, "Error converting ExtHeaderRecord to header")
            }
            InvalidBlockRange(start, end) => {
                write!(f, "Invalid block range: {} - {}", start, end)
            }
        }
    }
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::LockfileIoError(e) => write!(f, "Error reading lockfile: {e}"),
            Self::LockfileReadError => write!(f, "Epoch not found"),
        }
    }
}

impl From<std::io::Error> for SyncError {
    fn from(error: std::io::Error) -> Self {
        SyncError::LockfileIoError(error)
    }
}

impl From<StreamingFastProtosError> for EraValidateError {
    fn from(error: StreamingFastProtosError) -> Self {
        match error {
            StreamingFastProtosError::BlockConversionError => Self::HeaderDecodeError,
        }
    }
}
