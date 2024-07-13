use std::path::Path;

use ethportal_api::types::execution::accumulator::HeaderRecord;
use tree_hash::TreeHash;
use trin_validation::accumulator::PreMergeAccumulator;

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
/// * `premerge_accumulator`- An instance of [`PreMergeAccumulator`] which is a representation of a file that maintains a record of historical epoch
/// it is used to verify canonical-ness of headers accumulated from the `blocks`
/// * `start_epoch` -  The epoch number that all the first 8192 blocks are set located
/// * `end_epoch` -  The epoch number that all the last 8192 blocks are located
/// * `use_lock` - when set to true, uses the lockfile to store already processed blocks. True by default
pub fn era_validate(
    mut headers: Vec<ExtHeaderRecord>,
    premerge_accumulator: PreMergeAccumulator,
    start_epoch: usize,
    end_epoch: Option<usize>,
    use_lock: bool,
) -> Result<Vec<usize>, EraValidateError> {
    let end_epoch = end_epoch.unwrap_or(start_epoch + 1);

    // Ensure start epoch is less than end epoch
    if start_epoch >= end_epoch {
        Err(EraValidateError::EndEpochLessThanStartEpoch)?;
    }

    let mut validated_epochs = Vec::new();
    for epoch in start_epoch..end_epoch {
        // checks if epoch was already synced form lockfile.
        if use_lock {
            match check_sync_state(
                Path::new("./lockfile.json"),
                epoch.to_string(),
                premerge_accumulator.historical_epochs[epoch].0,
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
        }
        let epoch_headers: Vec<ExtHeaderRecord> = headers.drain(0..MAX_EPOCH_SIZE).collect();
        let root = process_headers(epoch_headers, epoch, &premerge_accumulator)?;
        validated_epochs.push(epoch);
        // stores the validated epoch into lockfile to avoid validating again and keeping a concise state
        if use_lock {
            match store_last_state(Path::new("./lockfile.json"), LockEntry::new(&epoch, root)) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("error: {}", e);
                    return Err(EraValidateError::EpochAccumulatorError);
                }
            }
        }
    }

    Ok(validated_epochs)
}

/// takes 8192 block headers and checks if they consist in a valid epoch.
///
/// An epoch must respect the order of blocks, i.e., block numbers for epoch
/// 0 must start from block 0 to block 8191.
///
/// headers can only be validated for now against epochs before The Merge.
/// All pre-merge blocks (which are numbered before [`FINAL_EPOCH`]), are validated using
/// the [Header Accumulator](https://github.com/ethereum/portal-network-specs/blob/8ad5bc33cb0d4485d2eab73bf2decc43e7566a8f/history-network.md#the-header-accumulator)
///
/// For block post merge, the sync-committee should be used to validate block headers   
/// in the canonical blockchain. So this function is not useful for those.
fn process_headers(
    mut headers: Vec<ExtHeaderRecord>,
    epoch: usize,
    pre_merge_accumulator: &PreMergeAccumulator,
) -> Result<[u8; 32], EraValidateError> {
    if headers.len() != MAX_EPOCH_SIZE {
        Err(EraValidateError::InvalidEpochLength)?;
    }
    if headers[0].block_number % MAX_EPOCH_SIZE as u64 != 0 {
        Err(EraValidateError::InvalidEpochStart)?;
    }

    if epoch > FINAL_EPOCH {
        log::warn!(
            "the blocks from this epoch are not being validated since they are post merge.
        For post merge blocks, use the sync-committee subprotocol"
        );
        headers.retain(|header: &ExtHeaderRecord| header.block_number < MERGE_BLOCK);
    }

    let header_records: Vec<HeaderRecord> = headers.into_iter().map(HeaderRecord::from).collect();
    let epoch_accumulator = compute_epoch_accumulator(&header_records)?;

    // Return an error if the epoch accumulator does not match the pre-merge accumulator
    let root: [u8; 32] = epoch_accumulator.tree_hash_root().0;
    let valid_root = pre_merge_accumulator.historical_epochs[epoch].0;
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
//     master_accumulator: PreMergeAccumulator,
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
