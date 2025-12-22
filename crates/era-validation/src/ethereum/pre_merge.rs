// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use alloy_primitives::FixedBytes;
use ethportal_api::types::execution::accumulator::EpochAccumulator;
use tree_hash::TreeHash;
use validation::{HistoricalEpochRoots, PreMergeAccumulator};

use crate::{
    error::{EraValidationError, EthereumPreMergeError},
    ethereum::types::{Epoch, FINAL_EPOCH},
    traits::EraValidationContext,
    types::EpochNumber,
};

/// a pre-merge ethereum validator that validates the era using historical roots. pre-merge
/// ethereum does not have a
/// consensus source of truth for historical data. we use a merkle tree to commit to the block
/// hashes. ethereum eras are defined as 8192, so we use that as the epoch length, i.e.
/// the number of values we commit to with a merkle tree. this yields a tree depth of 13. this
/// is the same datastructure as used by portal network, i.e. header accumulator. the
/// validator expects the era which is being verified and the corresponding block hashes. it checks
/// the tree hash root of the block hashes against precomputed historical roots for the era.
pub struct EthereumPreMergeValidator {
    /// historical epoch roots for pre-merge ethereum
    pub historical_roots: HistoricalEpochRoots,
}

impl EthereumPreMergeValidator {
    /// creates a new pre-merge ethereum validator.
    pub fn new(historical_roots: HistoricalEpochRoots) -> Self {
        Self { historical_roots }
    }

    /// validates the era using the historical roots.
    ///
    /// input: (era_number, block_hashes), where era_number is the era to validate and block_hashes
    /// is a vector of the block hashes for that era.
    pub fn validate_era(
        &self,
        input: (EpochNumber, EpochAccumulator),
    ) -> Result<(), EthereumPreMergeError> {
        self.historical_roots.validate_era(input)
    }

    /// validates many epochs against a header accumulator
    ///
    /// # Arguments
    ///
    /// * `epochs`-  an array of [`Epoch`].
    pub fn validate_eras(
        &self,
        epochs: &[&Epoch],
    ) -> Result<Vec<FixedBytes<32>>, EraValidationError> {
        let mut validated_epochs = Vec::new();
        for epoch in epochs {
            let root = self.validate_single_epoch(epoch)?;
            validated_epochs.push(root);
        }

        Ok(validated_epochs)
    }

    /// takes an epoch and validates against header accumulators
    ///
    /// epochs can only be validated for now against epochs before the merge.
    /// all pre-merge blocks (which are numbered before [`FINAL_EPOCH`]), are validated using
    /// the [header accumulator](https://github.com/ethereum/portal-network-specs/blob/8ad5bc33cb0d4485d2eab73bf2decc43e7566a8f/history-network.md#the-header-accumulator)
    ///
    /// for block post merge, the sync-committee should be used to validate block headers
    /// in the canonical blockchain. so this function is not useful for those.
    pub fn validate_single_epoch(
        &self,
        epoch: &Epoch,
    ) -> Result<FixedBytes<32>, EraValidationError> {
        if usize::from(epoch.number()) > FINAL_EPOCH {
            return Err(EraValidationError::EpochPostMerge(epoch.number()));
        }

        let header_records: Vec<_> = epoch.iter().cloned().collect();
        let epoch_accumulator = EpochAccumulator::from(header_records);

        let root = epoch_accumulator.tree_hash_root();

        let epoch_idx = usize::from(epoch.number());
        if epoch_idx >= self.historical_roots.len() {
            return Err(EraValidationError::EthereumPreMerge(
                EthereumPreMergeError::EpochOutOfBounds {
                    epoch: epoch.number(),
                    max_epoch: EpochNumber::from(self.historical_roots.len().saturating_sub(1)),
                },
            ));
        }
        let valid_root = self.historical_roots[epoch_idx];

        if root == valid_root {
            Ok(root)
        } else {
            tracing::error!(
                "the valid hash is: {:?} and the provided hash was: {:?}",
                valid_root,
                root
            );
            Err(EraValidationError::EraAccumulatorMismatch)
        }
    }
}

impl Default for EthereumPreMergeValidator {
    fn default() -> Self {
        PreMergeAccumulator::default().into()
    }
}

impl From<PreMergeAccumulator> for EthereumPreMergeValidator {
    fn from(value: PreMergeAccumulator) -> Self {
        Self {
            historical_roots: value.historical_epochs,
        }
    }
}

impl EraValidationContext for HistoricalEpochRoots {
    type EraInput = (EpochNumber, EpochAccumulator);
    type Error = EthereumPreMergeError;

    fn validate_era(&self, input: Self::EraInput) -> Result<(), Self::Error> {
        let era_number = input.0;
        let epoch_idx = usize::from(era_number);

        if epoch_idx >= self.len() {
            return Err(EthereumPreMergeError::EpochOutOfBounds {
                epoch: era_number,
                max_epoch: EpochNumber::from(self.len().saturating_sub(1)),
            });
        }

        let root = input.1.tree_hash_root();

        if root != self[epoch_idx] {
            return Err(EthereumPreMergeError::InvalidHistoricalRoot {
                era: era_number,
                expected: primitive_types::H256::from(self[epoch_idx].0),
                actual: primitive_types::H256::from(root.0),
            });
        }
        Ok(())
    }
}
