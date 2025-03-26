// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0
use crate::{impls::common::*, traits::EraValidationContext};
use merkle_proof::MerkleTree;
use primitive_types::H256;
use thiserror::Error;
use types::{BeaconBlock, MainnetEthSpec};
use alloy_primitives::FixedBytes;

#[derive(Error, Debug)]
pub enum EthereumPostMergeError {
    #[error("Number of execution block hashes must match the number of beacon blocks")]
    MismatchedBlockCount,
    #[error("Execution block hash mismatch: expected {expected:?}, got {actual:?}")]
    ExecutionBlockHashMismatch {
        expected: Option<H256>,
        actual: Option<H256>,
    },
    #[error("Invalid era start: slot {0} is not a multiple of 8192")]
    InvalidEraStart(u64),
    #[error("Invalid block summary root for era {era}: expected {expected}, got {actual}")]
    InvalidBlockSummaryRoot {
        era: usize,
        expected: H256,
        actual: H256,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EthereumHistoricalRoots(pub Vec<H256>);

/// A validator for Ethereum post-merge, pre-Capella blocks. It uses historical roots for
/// validation. The validator consumes an era of beacon blocks and the corresponding execution
/// blocks. It checks that the execution block hashes match the execution payloads in the beacon
/// blocks and that the tree hash root of the beacon blocks matches the historical root for the
/// era.
pub struct EthereumPostMergeValidator {
    pub historical_roots: EthereumHistoricalRoots, 
}

impl EthereumPostMergeValidator {
    /// Creates a new Ethereum post-merge validator.
    pub fn new(historical_roots: EthereumHistoricalRoots) -> Self {
        Self { historical_roots }
    }

    /// Validates the post-merge, pre-Capella era using the historical roots.
    ///
    /// input: (execution_block_hashes, beacon_blocks). execution_block_hashes is a vector of
    /// optional execution block hashes, it is optional because not all beacon blocks have an
    /// execution payload. beacon_blocks is a vector of beacon blocks for the era. It is expected
    /// that the execution_block_hash correspond one-to-one with the beacon_blocks. 
    pub fn validate_era(
        &self,
        input: (Vec<Option<H256>>, Vec<BeaconBlock<MainnetEthSpec>>),
    ) -> Result<(), EthereumPostMergeError> {
        self.historical_roots.validate_era(input)
    }
}

impl EraValidationContext for EthereumHistoricalRoots {
    type EraInput = (Vec<Option<H256>>, Vec<BeaconBlock<MainnetEthSpec>>);
    type EraOutput = Result<(), EthereumPostMergeError>;

    fn validate_era(&self, input: Self::EraInput) -> Self::EraOutput {
        let exec_hashes = input.0;
        let blocks = input.1;

        if blocks.len() != exec_hashes.len() {
            return Err(EthereumPostMergeError::MismatchedBlockCount);
        }

        for (block, expected_exec_hash) in blocks.iter().zip(exec_hashes.iter()) {
            // Check that the execution block hash matches the expected hash from the beacon block
            // execution payload, if there is one.
            match get_execution_payload_block_hash(block) {
                Some(execution_block_hash) => {
                    // Compare the block hash from the execution payload to the provided hash.
                    let actual_hash = Some(execution_block_hash);
                    if Some(actual_hash) != Some(*expected_exec_hash) {
                        return Err(EthereumPostMergeError::ExecutionBlockHashMismatch {
                            expected: *expected_exec_hash,
                            actual: actual_hash,
                        });
                    }
                }
                None => {
                    // If there's no execution payload, make sure no hash was provided.
                    if expected_exec_hash.is_some() {
                        return Err(EthereumPostMergeError::ExecutionBlockHashMismatch {
                            expected: None,
                            actual: *expected_exec_hash,
                        });
                    }
                }
            }
        }

        // Get era number from the slot of the first block: era = slot / 8192. Return an error if
        // not an even multiple of 8192.
        let era = blocks[0].slot() / 8192;
        if blocks[0].slot() % 8192 != 0 {
            return Err(EthereumPostMergeError::InvalidEraStart(
                blocks[0].slot().into(),
            ));
        }

        // Calculate the beacon block roots for each beacon block in the era.
        let mut roots: Vec<FixedBytes<32>> = Vec::new();
        for block in &blocks {
            let root = compute_tree_hash_root(block);
            roots.push(root.0.into());
        }

        // Calculate the tree hash root of the beacon block roots and compare against the
        // historical_summary.block_summary_root for the era.
        let beacon_block_roots_tree_hash_root = MerkleTree::create(roots.as_slice(), 13).hash();

        let true_root = self.0[usize::from(era)];

        if beacon_block_roots_tree_hash_root != FixedBytes::<32>::from(true_root.0) {
            return Err(EthereumPostMergeError::InvalidBlockSummaryRoot {
                era: usize::from(era),
                expected: true_root,
                actual: beacon_block_roots_tree_hash_root.0.into(),
            });
        }

        Ok(())
    }
}
