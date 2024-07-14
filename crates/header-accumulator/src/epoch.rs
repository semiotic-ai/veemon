use ethportal_api::types::execution::accumulator::{EpochAccumulator, HeaderRecord};

use crate::errors::EraValidateError;

pub const MAX_EPOCH_SIZE: usize = 8192;
pub const FINAL_EPOCH: usize = 1896;
pub const MERGE_BLOCK: u64 = 15537394;

pub fn compute_epoch_accumulator(
    header_records: &Vec<HeaderRecord>,
) -> Result<EpochAccumulator, EraValidateError> {
    if header_records.len() > MAX_EPOCH_SIZE {
        Err(EraValidateError::TooManyHeaderRecords)?;
    }

    let mut epoch_accumulator =
        EpochAccumulator::new(Vec::new()).map_err(|_| EraValidateError::EpochAccumulatorError)?;
    for header_record in header_records {
        let _ = epoch_accumulator.push(*header_record);
    }
    Ok(epoch_accumulator)
}
