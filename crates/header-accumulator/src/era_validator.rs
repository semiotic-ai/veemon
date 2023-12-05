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
) -> Result<(), EraValidateError> {
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

    for epoch in start_epoch..end_epoch {
        let start_100_block = epoch * MAX_EPOCH_SIZE - (epoch * MAX_EPOCH_SIZE % 100);
        let end_100_block =
            (epoch + 1) * MAX_EPOCH_SIZE + (100 - ((epoch + 1) * MAX_EPOCH_SIZE % 100));

        let mut blocks = extract_100_blocks(directory, start_100_block, end_100_block)?;

        if epoch < FINAL_EPOCH {
            blocks = blocks[0..MAX_EPOCH_SIZE].to_vec();
        } else {
            blocks = blocks[0..MERGE_BLOCK].to_vec();
        }

        let header_records = decode_header_records(&blocks)?;
        let epoch_accumulator = compute_epoch_accumulator(header_records)?;

        // Return an error if the epoch accumulator does not match the master accumulator
        if epoch_accumulator.tree_hash_root().0 != master_accumulator.historical_epochs[epoch].0 {
            Err(EraValidateError::EraAccumulatorMismatch)?;
        }

        println!("Epoch {} validated successfully", epoch);
    }

    Ok(())
}
