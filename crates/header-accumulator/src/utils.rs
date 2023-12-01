use decoder::{decode_flat_files, protos::block::Block};
use ethportal_api::types::execution::accumulator::{EpochAccumulator, HeaderRecord};
use primitive_types::{H256, U256};
use tree_hash::Hash256;

use crate::errors::EraValidateError;

pub const MAX_EPOCH_SIZE: usize = 8192;
pub const FINAL_EPOCH: usize = 01895;
pub const MERGE_BLOCK: usize = 15537394;

pub fn extract_100_blocks(
    directory: &String,
    start_100_block: usize,
    end_100_block: usize,
) -> Result<Vec<Block>, EraValidateError> {
    let mut blocks: Vec<Block> = Vec::new();
    for block_number in (start_100_block..end_100_block).step_by(100) {
        let block_file_name = directory.to_owned() + &format!("/{:010}.dbin", block_number);
        println!("Reading block file {}", block_file_name);
        let block = &decode_flat_files(&block_file_name, None, None)
            .map_err(|_| EraValidateError::FlatFileDecodeError)?;
        blocks.extend(block.clone());
    }
    Ok(blocks)
}

pub fn decode_header_records(blocks: &Vec<Block>) -> Result<Vec<HeaderRecord>, EraValidateError> {
    let mut header_records = Vec::<HeaderRecord>::new();
    for block in blocks {
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
            )
            .map_err(|_| EraValidateError::HeaderDecodeError)?,
        };
        header_records.push(header_record);
    }

    Ok(header_records)
}

pub fn compute_epoch_accumulator(
    header_records: Vec<HeaderRecord>,
) -> Result<EpochAccumulator, EraValidateError> {
    if header_records.len() > MAX_EPOCH_SIZE {
        Err(EraValidateError::TooManyHeaderRecords)?;
    }

    let mut epoch_accumulator =
        EpochAccumulator::new(Vec::new()).map_err(|_| EraValidateError::EpochAccumulatorError)?;
    for header_record in header_records {
        let _ = epoch_accumulator.push(header_record);
    }
    Ok(epoch_accumulator)
}
