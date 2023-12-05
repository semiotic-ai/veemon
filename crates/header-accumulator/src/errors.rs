use std::fmt;

#[derive(Debug)]
pub enum EraValidateError {
    TooManyHeaderRecords,
    InvalidMasterAccumulatorFile,
    HeaderDecodeError,
    FlatFileDecodeError,
    EraAccumulatorMismatch,
    EpochAccumulatorError,
    ProofGenerationFailure,
}
impl std::error::Error for EraValidateError {}

impl fmt::Display for EraValidateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EraValidateError::TooManyHeaderRecords => write!(f, "Too many header records"),
            EraValidateError::InvalidMasterAccumulatorFile => {
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
        }
    }
}
