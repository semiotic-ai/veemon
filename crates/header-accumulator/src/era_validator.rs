use std::fmt;
use decoder::decode_flat_files;
use ethportal_api::types::execution::accumulator::{EpochAccumulator, HeaderRecord};
use primitive_types::{H256 as Hash256, U256};
use tree_hash::TreeHash;
use trin_validation::accumulator::MasterAccumulator;

const MAX_EPOCH_SIZE: usize = 8192;

fn compute_epoch_accumulator(header_records: Vec<HeaderRecord>) -> Result<EpochAccumulator, EraValidateError> {
    if header_records.len() > MAX_EPOCH_SIZE {
        Err(EraValidateError::TooManyHeaderRecords)?;
    }

    let mut epoch_accumulator = EpochAccumulator::new(Vec::new()).map_err(|_| EraValidateError::EpochAccumulatorError)?;
    for header_record in header_records {
        let _ = epoch_accumulator.push(header_record);
    }
    Ok(epoch_accumulator)
}

pub fn era_validate(directory: &String, master_accumulator_file: Option<&String>, start_epoch: usize, end_epoch: Option<usize>) -> Result<(), EraValidateError> {
    // Load master accumulator if available, otherwise use default from Prortal Network
    let master_accumulator = match master_accumulator_file {
        Some(master_accumulator_file) => {
            MasterAccumulator::try_from_file(master_accumulator_file.into()).map_err(|_| EraValidateError::InvalidMasterAccumulatorFile)?
        }
        None => {
            MasterAccumulator::default()
        }
    };

    let end_epoch = match end_epoch  {
        Some(end_epoch) => end_epoch,
        None => start_epoch+1
    };
    // Load blocks from flat files
    let blocks = decode_flat_files(directory, None, None).map_err(|_| EraValidateError::FlatFileDecodeError)?;
    
    for epoch in start_epoch..end_epoch{
        let start_block_number = epoch * MAX_EPOCH_SIZE;
        let end_block_number = (epoch + 1) * MAX_EPOCH_SIZE;
        if end_block_number > blocks.len() {
            Err(EraValidateError::EndEraExceedsAvailableBlocks)?;
        }
        let mut block_number = start_block_number;
        let mut header_records = Vec::<HeaderRecord>::new();
        while block_number < end_block_number {
            let block = blocks
                .iter()
                .find(|&b| b.number == block_number as u64)
                .ok_or(EraValidateError::MissingBlock)?;
            
            let header_record = HeaderRecord {
                block_hash: Hash256::from_slice(block.hash.as_slice()),
                total_difficulty: U256::try_from(
                    block
                        .header
                        .total_difficulty
                        .as_ref()
                        .ok_or(EraValidateError::HeaderDecodeError)?
                        .bytes
                        .as_slice(),
                ).map_err(|_| EraValidateError::HeaderDecodeError)?,
            };
            header_records.push(header_record);
            block_number += 1;
        }

        let epoch_accumulator = compute_epoch_accumulator(header_records)?;

        // Return an error if the epoch accumulator does not match the master accumulator
        if epoch_accumulator.tree_hash_root().0 != master_accumulator.historical_epochs[0].0 {
            Err(EraValidateError::EraAccumulatorMismatch)?;
        }

        println!("Epoch {} validated successfully", epoch);
    }

    Ok(())
}

// Error definitions
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