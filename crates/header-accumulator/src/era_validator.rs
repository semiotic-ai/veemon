// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use ethportal_api::types::execution::accumulator::EpochAccumulator;
use tree_hash::{Hash256, TreeHash};
use trin_validation::accumulator::{HistoricalEpochRoots, PreMergeAccumulator};

use crate::{
    epoch::{Epoch, FINAL_EPOCH},
    errors::EraValidateError,
};

/// Contains a list with length 1896 with hashes for each epoch
pub struct EraValidator {
    historical_epochs: HistoricalEpochRoots,
}

impl Default for EraValidator {
    fn default() -> Self {
        PreMergeAccumulator::default().into()
    }
}

impl From<PreMergeAccumulator> for EraValidator {
    fn from(value: PreMergeAccumulator) -> Self {
        Self {
            historical_epochs: value.historical_epochs,
        }
    }
}

impl EraValidator {
    /// Validates many epochs against a header accumulator
    ///
    /// # Arguments
    ///
    /// * `epochs`-  An array of [`Epoch`].
    pub fn validate_eras(&self, epochs: &[&Epoch]) -> Result<Vec<Hash256>, EraValidateError> {
        let mut validated_epochs = Vec::new();
        for epoch in epochs {
            let root = self.validate_era(epoch)?;
            validated_epochs.push(root);
        }

        Ok(validated_epochs)
    }

    /// Takes an Epoch and validates against Header Accumulators
    ///
    /// Epochs can only be validated for now against epochs before The Merge.
    /// All pre-merge blocks (which are numbered before [`FINAL_EPOCH`]), are validated using
    /// the [Header Accumulator](https://github.com/ethereum/portal-network-specs/blob/8ad5bc33cb0d4485d2eab73bf2decc43e7566a8f/history-network.md#the-header-accumulator)
    ///
    /// For block post merge, the sync-committee should be used to validate block headers
    /// in the canonical blockchain. So this function is not useful for those.
    pub fn validate_era(&self, epoch: &Epoch) -> Result<Hash256, EraValidateError> {
        if epoch.number() > FINAL_EPOCH {
            return Err(EraValidateError::EpochPostMerge(epoch.number()));
        }

        let header_records: Vec<_> = epoch.iter().cloned().collect();
        let epoch_accumulator = EpochAccumulator::from(header_records);

        let root = epoch_accumulator.tree_hash_root();
        let valid_root = self.historical_epochs[epoch.number()];

        if root == valid_root {
            Ok(root)
        } else {
            tracing::error!(
                "the valid hash is: {:?} and the provided hash was: {:?}",
                valid_root,
                root
            );
            Err(EraValidateError::EraAccumulatorMismatch)
        }
    }
}
