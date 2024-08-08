use std::path::Path;

use ethportal_api::types::execution::accumulator::{EpochAccumulator, HeaderRecord};
use tree_hash::TreeHash;
use trin_validation::accumulator::PreMergeAccumulator;

use crate::{
    epoch::{FINAL_EPOCH, MAX_EPOCH_SIZE, MERGE_BLOCK},
    errors::{EraValidateError, HeaderAccumulatorError},
    sync::{Lock, LockEntry},
    types::ExtHeaderRecord,
};

pub trait EraValidator {
    type Error;

    /// Validates many headers against a header accumulator
    ///
    /// It also keeps a record in `lockfile.json` of the validated epochs to skip them
    ///
    /// # Arguments
    ///
    /// * `headers`-  A mutable vector of [`ExtHeaderRecord`]. The Vector can be any size,
    ///   however, it must be in chunks of 8192 blocks to work properly to function without error
    /// * `start_epoch` -  The epoch number that all the first 8192 blocks are set located
    /// * `end_epoch` -  The epoch number that all the last 8192 blocks are located
    /// * `use_lock` - when set to true, uses the lockfile to store already processed blocks. True by default
    fn era_validate(
        &self,
        headers: Vec<ExtHeaderRecord>,
        start_epoch: usize,
        end_epoch: Option<usize>,
        use_lock: bool,
    ) -> Result<Vec<usize>, Self::Error>;

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
        &self,
        headers: Vec<ExtHeaderRecord>,
        epoch: usize,
    ) -> Result<[u8; 32], Self::Error>;
}

impl EraValidator for PreMergeAccumulator {
    type Error = HeaderAccumulatorError;

    fn era_validate(
        &self,
        mut headers: Vec<ExtHeaderRecord>,
        start_epoch: usize,
        end_epoch: Option<usize>,
        use_lock: bool,
    ) -> Result<Vec<usize>, Self::Error> {
        let end_epoch = end_epoch.unwrap_or(start_epoch + 1);

        // Ensure start epoch is less than end epoch
        if start_epoch >= end_epoch {
            Err(EraValidateError::EndEpochLessThanStartEpoch)?;
        }

        let mut validated_epochs = Vec::new();
        for epoch in start_epoch..end_epoch {
            // checks if epoch was already synced form lockfile.
            if use_lock {
                let file_path = Path::new("./lockfile.json");
                let lock_file = Lock::from_file(file_path)?;

                match lock_file.check_sync_state(file_path, epoch, self.historical_epochs[epoch].0)
                {
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
                            Err(EraValidateError::EpochAccumulatorError.into())
                        }
                    }
                }
            }
            let epoch_headers: Vec<ExtHeaderRecord> = headers.drain(0..MAX_EPOCH_SIZE).collect();
            let root = self.process_headers(epoch_headers, epoch)?;
            validated_epochs.push(epoch);

            // stores the validated epoch into lockfile to avoid validating again and keeping a concise state
            if use_lock {
                let path = Path::new("./lockfile.json");
                let mut lock_file = Lock::from_file(path)?;
                lock_file.update(LockEntry::new(&epoch, root));

                match lock_file.store_last_state(path) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("error: {}", e);
                        return Err(EraValidateError::EpochAccumulatorError.into());
                    }
                }
            }
        }

        Ok(validated_epochs)
    }

    fn process_headers(
        &self,
        mut headers: Vec<ExtHeaderRecord>,
        epoch: usize,
    ) -> Result<[u8; 32], Self::Error> {
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

        let header_records: Vec<_> = headers.into_iter().map(HeaderRecord::from).collect();
        let epoch_accumulator = EpochAccumulator::from(header_records);

        let root: [u8; 32] = epoch_accumulator.tree_hash_root().0;
        let valid_root = self.historical_epochs[epoch].0;

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
}
