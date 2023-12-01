use decoder::{decode_flat_files, protos::block::Block};
use ethportal_api::types::execution::accumulator::{EpochAccumulator, HeaderRecord};
use primitive_types::{H256 as Hash256, U256};
use trin_validation::accumulator::MasterAccumulator;
use tree_hash::TreeHash;
// use decoder::
use crate::errors::EraValidateError;

const MAX_EPOCH_SIZE: usize = 8192;

fn decode_header_records(blocks: &Vec<Block>, start_block: usize, end_block: usize) -> Result<Vec<HeaderRecord>, EraValidateError> {

    let mut block_number = start_block;
    let mut header_records = Vec::<HeaderRecord>::new();
    while block_number < end_block {
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

    Ok(header_records)
}

pub fn compute_epoch_accumulator(header_records: Vec<HeaderRecord>) -> Result<EpochAccumulator, EraValidateError> {
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

        let header_records = decode_header_records(&blocks, start_block_number, end_block_number)?;
        let epoch_accumulator = compute_epoch_accumulator(header_records)?;

        // Return an error if the epoch accumulator does not match the master accumulator
        if epoch_accumulator.tree_hash_root().0 != master_accumulator.historical_epochs[0].0 {
            Err(EraValidateError::EraAccumulatorMismatch)?;
        }

        println!("Epoch {} validated successfully", epoch);
    }

    Ok(())
}
