use std::path::Path;

use decoder::{decode_flat_files, sf::ethereum::r#type::v2::Block};
use header_accumulator::{
    errors::EraValidateError,
    sync::{check_sync_state, store_last_state, LockEntry},
    utils::{compute_epoch_accumulator, decode_header_records},
};
use tree_hash::TreeHash;

use trin_validation::accumulator::MasterAccumulator;

pub const MAX_EPOCH_SIZE: usize = 8192;
pub const FINAL_EPOCH: usize = 01896;
pub const MERGE_BLOCK: usize = 15537394;

// /// Validates an era against a header accumulator.
// pub fn era_validate(
//     directory: &String,
//     master_accumulator_file: Option<&String>,
//     start_epoch: usize,
//     end_epoch: Option<usize>,
// ) -> Result<Vec<usize>, EraValidateError> {
//     // Load master accumulator if available, otherwise use default from Portal Network
//     let master_accumulator = match master_accumulator_file {
//         Some(master_accumulator_file) => {
//             master_accumulator_file::try_from_file(master_accumulator_file.into())
//                 .map_err(|_| EraValidateError::InvalidMasterAccumulatorFile)?
//         }
//         None => master_accumulator_file::default(),
//     };

//     let end_epoch = match end_epoch {
//         Some(end_epoch) => end_epoch,
//         None => start_epoch + 1,
//     };
//     // Ensure start epoch is less than end epoch
//     if start_epoch >= end_epoch {
//         Err(EraValidateError::EndEpochLessThanStartEpoch)?;
//     }

//     let mut validated_epochs = Vec::new();
//     for epoch in start_epoch..end_epoch {
//         // checkes if epoch was already synced form lockfile.
//         match check_sync_state(
//             Path::new("./lockfile.json"),
//             epoch.to_string(),
//             master_accumulator.historical_epochs[epoch].0,
//         ) {
//             Ok(true) => {
//                 log::info!("Skipping, epoch already synced: {}", epoch);
//                 continue;
//             }
//             Ok(false) => {
//                 log::info!("syncing new epoch: {}", epoch);
//             }
//             Err(_) => return Err(EraValidateError::EpochAccumulatorError),
//         }

//         let root = process_epoch_from_directory(epoch, directory, master_accumulator.clone())?;
//         validated_epochs.push(epoch);
//         // stores the validated epoch into lockfile to avoid validating again and keeping a concise state
//         match store_last_state(Path::new("./lockfile.json"), LockEntry::new(&epoch, root)) {
//             Ok(_) => {}
//             Err(_) => return Err(EraValidateError::EpochAccumulatorError),
//         }
//     }

//     Ok(validated_epochs)
// }

fn process_epoch_from_directory(
    epoch: usize,
    directory: &String,
    master_accumulator: MasterAccumulator,
) -> Result<[u8; 32], EraValidateError> {
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

    //compute_epoch_accumulator Return an error if the epoch accumulator does not match the master accumulator
    let root: [u8; 32] = epoch_accumulator.tree_hash_root().0;
    if root != master_accumulator.historical_epochs[epoch].0 {
        Err(EraValidateError::EraAccumulatorMismatch)?;
    }

    Ok(root)
}

pub fn extract_100_blocks(
    directory: &String,
    start_block: usize,
    end_block: usize,
) -> Result<Vec<Block>, EraValidateError> {
    // Flat files are stored in 100 block files
    // So we need to find the 100 block file that contains the start block and the 100 block file that contains the end block
    let start_100_block = (start_block / 100) * 100;
    let end_100_block = (((end_block as f32) / 100.0).ceil() as usize) * 100;

    let mut blocks: Vec<Block> = Vec::new();
    for block_number in (start_100_block..end_100_block).step_by(100) {
        let block_file_name = directory.to_owned() + &format!("/{:010}.dbin", block_number);
        let block = &decode_flat_files(block_file_name, None, None)
            .map_err(|_| EraValidateError::FlatFileDecodeError)?;
        blocks.extend(block.clone());
    }

    // Return only the requested blocks
    Ok(blocks[start_block - start_100_block..end_block - start_100_block].to_vec())
}
