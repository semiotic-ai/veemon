use std::{
    io::{BufRead, Read, Write as StdWrite},
    path::Path,
};

use decoder::{
    headers::HeaderRecordWithNumber,
    sf::{self, ethereum::r#type::v2::Block},
};

use ethportal_api::types::execution::accumulator::HeaderRecord;
use primitive_types::{H256, U256};
use tree_hash::TreeHash;
use trin_validation::accumulator::MasterAccumulator;

use crate::{
    errors::EraValidateError,
    sync::{check_sync_state, store_last_state, LockEntry},
    utils::{
        compute_epoch_accumulator, decode_header_records, FINAL_EPOCH, MAX_EPOCH_SIZE, MERGE_BLOCK,
    },
};

/// Validates many blocks against a header accumulator
///
/// It also keeps a record in `lockfile.json` of the validated epochs to skip them
///
/// # Arguments
///
/// * `blocks`-  A mutable vector of blocks. The Vector can be any size, however, it must be in chunks of 8192 blocks
/// to function without error
/// * `master_accumulator_file`- An instance of `MasterAccumulator` which is a file that maintains a record of historical epoch
/// it is used to verify canonical-ness of headers accumulated from the `blocks`
/// * `start_epoch` -  The epoch number that all the first 8192 blocks are set located
/// * `end_epoch` -  The epoch number that all the last 8192 blocks are located
pub fn era_validate(
    mut blocks: Vec<sf::ethereum::r#type::v2::Block>,
    master_accumulator_file: Option<&String>,
    start_epoch: usize,
    end_epoch: Option<usize>,
) -> Result<Vec<usize>, EraValidateError> {
    // Load master accumulator if available, otherwise use default from Portal Network
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

    let mut validated_epochs = Vec::new();
    for epoch in start_epoch..=end_epoch {
        // checkes if epoch was already synced form lockfile.
        match check_sync_state(
            Path::new("./lockfile.json"),
            epoch.to_string(),
            master_accumulator.historical_epochs[epoch].0,
        ) {
            Ok(true) => {
                log::info!("Skipping, epoch already synced: {}", epoch);
                continue;
            }
            Ok(false) => {
                log::info!("syncing new epoch: {}", epoch);
            }
            Err(e) => {
                return {
                    log::error!("error: {}", e);
                    Err(EraValidateError::EpochAccumulatorError)
                }
            }
        }

        let epoch_blocks: Vec<Block> = blocks.drain(0..MAX_EPOCH_SIZE).collect();
        let root = process_blocks(epoch_blocks, epoch, &master_accumulator)?;
        validated_epochs.push(epoch);
        // stores the validated epoch into lockfile to avoid validating again and keeping a concise state
        match store_last_state(Path::new("./lockfile.json"), LockEntry::new(&epoch, root)) {
            Ok(_) => {}
            Err(e) => {
                log::error!("error: {}", e);
                return Err(EraValidateError::EpochAccumulatorError);
            }
        }
    }

    Ok(validated_epochs)
}

/// takes 8192 blocks and checks if they consist in a valid epoch
fn process_blocks(
    mut blocks: Vec<sf::ethereum::r#type::v2::Block>,
    epoch: usize,
    master_accumulator: &MasterAccumulator,
) -> Result<[u8; 32], EraValidateError> {
    if blocks.len() != MAX_EPOCH_SIZE {
        Err(EraValidateError::InvalidEpochLength)?;
    }

    if epoch > FINAL_EPOCH {
        blocks.retain(|block: &Block| block.number < MERGE_BLOCK);
    }

    let header_records = decode_header_records(blocks)?;
    let epoch_accumulator = compute_epoch_accumulator(&header_records)?;

    // Return an error if the epoch accumulator does not match the master accumulator
    let root: [u8; 32] = epoch_accumulator.tree_hash_root().0;
    let valid_root = master_accumulator.historical_epochs[epoch].0;
    if root != valid_root {
        log::error!(
            "the valid hash is: {:?} and the provided hash was: {:?}",
            valid_root,
            root
        );
        Err(EraValidateError::EraAccumulatorMismatch)?;
    }

    Ok(root)
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
