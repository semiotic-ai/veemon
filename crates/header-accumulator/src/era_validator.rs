use std::io::{BufRead, Read, Write as StdWrite};

use decoder::headers::HeaderRecordWithNumber;
use ethportal_api::types::execution::accumulator::HeaderRecord;
use primitive_types::{H256, U256};
use tree_hash::TreeHash;
use trin_validation::accumulator::MasterAccumulator;

use crate::{
    errors::EraValidateError,
    utils::{
        compute_epoch_accumulator, decode_header_records, extract_100_blocks, FINAL_EPOCH,
        MAX_EPOCH_SIZE, MERGE_BLOCK,
    },
};

pub fn era_validate(
    directory: &String,
    master_accumulator_file: Option<&String>,
    start_epoch: usize,
    end_epoch: Option<usize>,
) -> Result<Vec<usize>, EraValidateError> {
    // Load master accumulator if available, otherwise use default from Prortal Network
    let master_accumulator = match master_accumulator_file {
        Some(master_accumulator_file) => {
            MasterAccumulator::try_from_file(master_accumulator_file.into())
                .map_err(|_| EraValidateError::InvalidMasterAccumulatorFile)?
        }
        None => MasterAccumulator::default(),
    };

    let end_epoch = match end_epoch {
        Some(end_epoch) => end_epoch,
        None => start_epoch + 1,
    };
    // Ensure start epoch is less than end epoch
    if start_epoch >= end_epoch {
        Err(EraValidateError::EndEpochLessThanStartEpoch)?;
    }

    let mut validated_epoch = Vec::new();
    for epoch in start_epoch..end_epoch {
        process_epoch_from_directory(epoch, directory, master_accumulator.clone())?;
        validated_epoch.push(epoch);
    }

    Ok(validated_epoch)
}

fn process_epoch_from_directory(
    epoch: usize,
    directory: &String,
    master_accumulator: MasterAccumulator,
) -> Result<(), EraValidateError> {
    let start_100_block = epoch * MAX_EPOCH_SIZE;
    let end_100_block = (epoch + 1) * MAX_EPOCH_SIZE;

    let mut blocks = extract_100_blocks(directory, start_100_block, end_100_block)?;

    if epoch < FINAL_EPOCH {
        blocks = blocks[0..MAX_EPOCH_SIZE].to_vec();
    } else {
        blocks = blocks[0..MERGE_BLOCK].to_vec();
    }

    let header_records = decode_header_records(blocks)?;
    let epoch_accumulator = compute_epoch_accumulator(&header_records)?;

    // Return an error if the epoch accumulator does not match the master accumulator
    if epoch_accumulator.tree_hash_root().0 != master_accumulator.historical_epochs[epoch].0 {
        Err(EraValidateError::EraAccumulatorMismatch)?;
    }

    Ok(())
}

pub fn stream_validation<R: Read + BufRead, W: StdWrite>(
    master_accumulator: MasterAccumulator,
    mut reader: R,
    mut writer: W,
) -> Result<(), EraValidateError> {
    let mut header_records = Vec::new();
    let mut append_flag = false;
    let mut buf = String::new();

    while let Ok(hrwn) = receive_message(&mut reader) {
        buf.clear();
        if header_records.len() == 0 {
            if hrwn.block_number % MAX_EPOCH_SIZE as u64 == 0 {
                let epoch = hrwn.block_number as usize / MAX_EPOCH_SIZE;
                log::info!("Validating epoch: {}", epoch);
                append_flag = true;
            }
        }
        if append_flag == true {
            let header_record = HeaderRecord {
                block_hash: H256::from_slice(&hrwn.block_hash),
                total_difficulty: U256::try_from(hrwn.total_difficulty.as_slice())
                    .map_err(|_| EraValidateError::TotalDifficultyDecodeError)?,
            };
            header_records.push(header_record);
        }

        if header_records.len() == MAX_EPOCH_SIZE {
            let epoch = hrwn.block_number as usize / MAX_EPOCH_SIZE;
            let epoch_accumulator = compute_epoch_accumulator(&header_records)?;
            if epoch_accumulator.tree_hash_root().0 != master_accumulator.historical_epochs[epoch].0
            {
                Err(EraValidateError::EraAccumulatorMismatch)?;
            }
            log::info!("Validated epoch: {}", epoch);
            writer
                .write_all(format!("Validated epoch: {}\n", epoch).as_bytes())
                .map_err(|_| EraValidateError::JsonError)?;
            header_records.clear();
        }
    }
    log::info!("Read {} block headers from stdin", header_records.len());
    Ok(())
}

fn receive_message<R: Read>(reader: &mut R) -> Result<HeaderRecordWithNumber, bincode::Error> {
    let mut size_buf = [0u8; 4];
    if reader.read_exact(&mut size_buf).is_err() {
        return Err(Box::new(bincode::ErrorKind::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Failed to read size",
        ))));
    }
    let size = u32::from_be_bytes(size_buf) as usize;

    let mut buf = vec![0u8; size];
    reader.read_exact(&mut buf)?;
    bincode::deserialize(&buf)
}
