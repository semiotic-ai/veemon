// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0
use crate::traits::EraValidationContext;
use merkle_proof::MerkleTree;
use primitive_types::H256;
use thiserror::Error;

/// A Solana validator that validates the era using historical roots. Solana does not have a
/// consensus source of truth for historical data. We use a Merkle tree to commit to the block
/// hashes. Solana epochs are defined as 432,000 slots, so we use that as the epoch length, i.e.
/// the number of values we commit to with a Merkle tree. This yields a tree depth of 19. The
/// validator expects the era which is being verified and the corresponding block hashes. It checks
/// the tree hash root of the block hashes against precomputed historical roots for the era.
const SOLANA_EPOCH_LENGTH: usize = 432_000;
const SOLANA_HISTORICAL_TREE_DEPTH: usize = 19;

#[derive(Error, Debug)]
pub enum SolanaValidatorError {
    #[error("Number of execution block hashes must match the epoch length")]
    MismatchedBlockCount,
    #[error("Invalid historical root for era {era}: expected {expected}, got {actual}")]
    InvalidHistoricalRoot {
        era: usize,
        expected: H256,
        actual: H256,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolanaHistoricalRoots(pub Vec<H256>);

pub struct SolanaValidator {
    pub historical_roots: SolanaHistoricalRoots,
}

impl SolanaValidator {
    /// Creates a new Solana validator.
    pub fn new(historical_roots: SolanaHistoricalRoots) -> Self {
        Self { historical_roots }
    }

    /// Validates the era using the historical roots.
    pub fn validate_era(&self, input: (usize, Vec<H256>)) -> Result<(), SolanaValidatorError> {
        self.historical_roots.validate_era(input)
    }
}

impl EraValidationContext for SolanaHistoricalRoots {
    /// (era_number, block_hashes)
    type EraInput = (usize, Vec<H256>);
    type EraOutput = Result<(), SolanaValidatorError>;

    fn validate_era(&self, input: Self::EraInput) -> Self::EraOutput {
        let era_number = input.0;
        let block_roots = input.1;
        if block_roots.len() != SOLANA_EPOCH_LENGTH {
            return Err(SolanaValidatorError::MismatchedBlockCount);
        }

        let root = MerkleTree::create(block_roots.as_slice(), SOLANA_HISTORICAL_TREE_DEPTH).hash();

        // Check that root matches the expected historical root
        if root != self.0[era_number] {
            return Err(SolanaValidatorError::InvalidHistoricalRoot {
                era: era_number,
                expected: self.0[era_number],
                actual: root,
            });
        }
        Ok(())
    }
}
