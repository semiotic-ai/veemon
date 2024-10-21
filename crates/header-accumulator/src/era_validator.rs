use ethportal_api::types::execution::accumulator::{EpochAccumulator, HeaderRecord};
use tree_hash::TreeHash;
use trin_validation::accumulator::{HistoricalEpochRoots, PreMergeAccumulator};

use crate::{
    epoch::{Epoch, FINAL_EPOCH},
    errors::EraValidateError,
};

pub struct EraValidator {
    historical_epochs: HistoricalEpochRoots,
}

impl From<PreMergeAccumulator> for EraValidator {
    fn from(value: PreMergeAccumulator) -> Self {
        Self {
            historical_epochs: value.historical_epochs,
        }
    }
}

pub type RootHash = [u8; 32];

impl EraValidator {
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
    pub fn validate_eras(&self, epochs: &[&Epoch]) -> Result<Vec<RootHash>, EraValidateError> {
        let mut validated_epochs = Vec::new();
        for epoch in epochs {
            let root = self.validate_era(epoch)?;
            validated_epochs.push(root);
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
    pub fn validate_era(&self, epoch: &Epoch) -> Result<RootHash, EraValidateError> {
        if epoch.number() > FINAL_EPOCH {
            log::warn!(
                "the blocks from this epoch are not being validated since they are post merge.
            For post merge blocks, use the sync-committee subprotocol"
            );
            // TODO return error
        }

        let header_records: Vec<_> = epoch.iter().map(HeaderRecord::from).collect();
        let epoch_accumulator = EpochAccumulator::from(header_records);

        let root: [u8; 32] = epoch_accumulator.tree_hash_root().0;
        let valid_root = self.historical_epochs[epoch.number()].0;

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
