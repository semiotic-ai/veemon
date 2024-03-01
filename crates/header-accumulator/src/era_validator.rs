use std::path::Path;

use ethportal_api::types::execution::accumulator::HeaderRecord;

use tree_hash::TreeHash;
use trin_validation::accumulator::MasterAccumulator;

use crate::{
    errors::EraValidateError,
    sync::{check_sync_state, store_last_state, LockEntry},
    types::ExtHeaderRecord,
    utils::{compute_epoch_accumulator, FINAL_EPOCH, MAX_EPOCH_SIZE, MERGE_BLOCK},
};

/// Validates many headers against a header accumulator
///
/// It also keeps a record in `lockfile.json` of the validated epochs to skip them
///
/// # Arguments
///
/// * `headers`-  A mutable vector of [`ExtHeaderRecord`]. The Vector can be any size, however, it must be in chunks of 8192 blocks to work properly
/// to function without error
/// * `master_accumulator_file`- An instance of [`MasterAccumulator`] which is a file that maintains a record of historical epoch
/// it is used to verify canonical-ness of headers accumulated from the `blocks`
/// * `start_epoch` -  The epoch number that all the first 8192 blocks are set located
/// * `end_epoch` -  The epoch number that all the last 8192 blocks are located
pub fn era_validate(
    mut headers: Vec<ExtHeaderRecord>,
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
    for epoch in start_epoch..end_epoch {
        // checks if epoch was already synced form lockfile.
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

        let epoch_headers: Vec<ExtHeaderRecord> = headers.drain(0..MAX_EPOCH_SIZE).collect();
        let root = process_headers(epoch_headers, epoch, &master_accumulator)?;
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

/// takes 8192 block headers and checks if they consist in a valid epoch
fn process_headers(
    mut headers: Vec<ExtHeaderRecord>,
    epoch: usize,
    master_accumulator: &MasterAccumulator,
) -> Result<[u8; 32], EraValidateError> {
    if headers.len() != MAX_EPOCH_SIZE {
        Err(EraValidateError::InvalidEpochLength)?;
    }

    if epoch > FINAL_EPOCH {
        headers.retain(|header: &ExtHeaderRecord| header.block_number < MERGE_BLOCK);
    }

    let header_records: Vec<HeaderRecord> = headers
        .into_iter()
        .map(|ext_record| HeaderRecord::from(ext_record))
        .collect();
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

// TODO: move stream validation to be a functionality of flat_head
// pub fn stream_validation<R: Read + BufRead, W: StdWrite>(
//     master_accumulator: MasterAccumulator,
//     mut reader: R,
//     mut writer: W,
// ) -> Result<(), EraValidateError> {
//     let mut header_records = Vec::new();
//     let mut append_flag = false;
//     let mut buf = String::new();

//     while let Ok(hrwn) = receive_message(&mut reader) {
//         buf.clear();

//         log::info!("{:?}", hrwn.block_hash);
//         if header_records.len() == 0 {
//             if hrwn.block_number % MAX_EPOCH_SIZE as u64 == 0 {
//                 let epoch = hrwn.block_number as usize / MAX_EPOCH_SIZE;
//                 log::info!("Validating epoch: {}", epoch);
//                 append_flag = true;
//             }
//         }
//         if append_flag == true {
//             let header_record = HeaderRecord {
//                 block_hash: H256::from_slice(&hrwn.block_hash),
//                 total_difficulty: U256::try_from(hrwn.total_difficulty.as_slice())
//                     .map_err(|_| EraValidateError::TotalDifficultyDecodeError)?,
//             };
//             header_records.push(header_record);
//         }

//         if header_records.len() == MAX_EPOCH_SIZE {
//             let epoch = hrwn.block_number as usize / MAX_EPOCH_SIZE;
//             let epoch_accumulator = compute_epoch_accumulator(&header_records)?;
//             if epoch_accumulator.tree_hash_root().0 != master_accumulator.historical_epochs[epoch].0
//             {
//                 Err(EraValidateError::EraAccumulatorMismatch)?;
//             }
//             log::info!("Validated epoch: {}", epoch);
//             writer
//                 .write_all(format!("Validated epoch: {}\n", epoch).as_bytes())
//                 .map_err(|_| EraValidateError::JsonError)?;
//             header_records.clear();
//         }
//     }

//     log::info!("Read {} block headers from stdin", header_records.len());
//     Ok(())
// }

// TODO: this functionality should be moved to flat_head
// fn receive_message<R: Read>(reader: &mut R) -> Result<HeaderRecordWithNumber, Box<dyn Error>> {
//     let mut size_buf = [0u8; 4];
//     if reader.read_exact(&mut size_buf).is_err() {
//         return Err(Box::new(bincode::ErrorKind::Io(std::io::Error::new(
//             std::io::ErrorKind::UnexpectedEof,
//             "Failed to read size",
//         ))));
//     }

//     let size = u32::from_be_bytes(size_buf) as usize;
//     println!("size: {:?}", size);

//     let mut buf = vec![0u8; size];
//     reader.read_exact(&mut buf)?;
//     let hrwn: HeaderRecordWithNumber = bincode::deserialize(&buf)?;

//     println!(" decoding {:?}", hrwn);
//     Ok(hrwn)
// }

// Test function

//  TODO: move this test to flat_head
// #[cfg(test)]
// mod tests {
//     use std::{fs::File, io, path::PathBuf};

//     use super::*;

//     #[test]
//     fn test_receive_message_from_file() -> Result<(), Box<dyn Error>> {
//         let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
//         path.push("tests/ethereum_firehose_first_8200/0000000000.dbin"); // Adjust the path as needed

//         let file = File::open(path)?;
//         let mut reader = io::BufReader::new(file);

//         let result = receive_message(&mut reader)?;

//         println!("block: {:?}", result.block_hash);

//         Ok(())
//     }
// }
