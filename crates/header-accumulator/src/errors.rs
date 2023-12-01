use std::fmt;

#[derive(Debug)]
pub enum EraValidateError {
    TooManyHeaderRecords,
    InvalidMasterAccumulatorFile,
    MissingBlock,
    HeaderDecodeError,
    FlatFileDecodeError,
    EraAccumulatorMismatch,
    EndEraExceedsAvailableBlocks,
    EpochAccumulatorError,
}
impl std::error::Error for EraValidateError {}

impl fmt::Display for EraValidateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EraValidateError::TooManyHeaderRecords => write!(f, "Too many header records"),
            EraValidateError::InvalidMasterAccumulatorFile => write!(f, "Invalid master accumulator file"),
            EraValidateError::MissingBlock => write!(f, "Missing block in flat files directory"),
            EraValidateError::HeaderDecodeError => write!(f, "Error decoding header from flat files"),
            EraValidateError::FlatFileDecodeError => write!(f, "Error decoding flat files"),
            EraValidateError::EraAccumulatorMismatch => write!(f, "Era accumulator mismatch"),
            EraValidateError::EndEraExceedsAvailableBlocks => write!(f, "End era exceeds available blocks"),
            EraValidateError::EpochAccumulatorError => write!(f, "Error creating epoch accumulator"),
        }
    }
}