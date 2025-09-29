// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::traits::EraValidationContext;
use ethportal_api::types::execution::accumulator::EpochAccumulator;
use primitive_types::H256;
use thiserror::Error;
use tree_hash::TreeHash;
use validation::HistoricalEpochRoots;

#[derive(Error, Debug)]
pub enum EthereumPreMergeValidatorError {
    #[error("Number of execution block hashes must match the epoch length")]
    MismatchedBlockCount,
    #[error("Invalid historical root for era {era}: expected {expected}, got {actual}")]
    InvalidHistoricalRoot {
        era: usize,
        expected: H256,
        actual: H256,
    },
}

/// A pre-merge Ethereum validator that validates the era using historical roots. Pre-merge
/// Ethereum does not have a
/// consensus source of truth for historical data. We use a Merkle tree to commit to the block
/// hashes. Ethereum eras are defined as 8192, so we use that as the epoch length, i.e.
/// the number of values we commit to with a Merkle tree. This yields a tree depth of 13. This
/// is the same datastructure as used by Portal Network, i.e. Header Accumulator. The
/// validator expects the era which is being verified and the corresponding block hashes. It checks
/// the tree hash root of the block hashes against precomputed historical roots for the era.
pub struct EthereumPreMergeValidator {
    pub historical_roots: HistoricalEpochRoots,
}

impl EthereumPreMergeValidator {
    /// Creates a new pre-merge Ethereum validator.
    pub fn new(historical_roots: HistoricalEpochRoots) -> Self {
        Self { historical_roots }
    }

    /// Validates the era using the historical roots.
    ///
    /// input: (era_number, block_hashes), where era_number is the era to validate and block_hashes
    /// is a vector of the block hashes for that era.
    pub fn validate_era(
        &self,
        input: (usize, EpochAccumulator),
    ) -> Result<(), EthereumPreMergeValidatorError> {
        self.historical_roots.validate_era(input)
    }
}

impl EraValidationContext for HistoricalEpochRoots {
    type EraInput = (usize, EpochAccumulator);
    type EraOutput = Result<(), EthereumPreMergeValidatorError>;

    fn validate_era(&self, input: Self::EraInput) -> Self::EraOutput {
        let era_number = input.0;

        let root = input.1.tree_hash_root();

        // Check that root matches the expected historical root
        if root != self[era_number] {
            return Err(EthereumPreMergeValidatorError::InvalidHistoricalRoot {
                era: era_number,
                expected: H256::from(self[era_number].0),
                actual: H256::from(root.0),
            });
        }
        Ok(())
    }
}
