// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{EthereumPosEraError, EthereumPostCapellaError},
    ethereum::{common::*, types::MAX_EPOCH_SIZE},
    traits::EraValidationContext,
    types::{EraNumber, SlotNumber},
};
use alloy_primitives::FixedBytes;
use merkle_proof::MerkleTree;
use primitive_types::H256;
use types::{BeaconBlock, MainnetEthSpec};
use validation::constants::CAPELLA_FORK_EPOCH;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EthereumBlockSummaryRoots(pub Vec<H256>);

/// a validator for ethereum post-capella blocks. it uses the block summary roots from historical summaries for validation. the
/// validator consumes an era of beacon blocks and the corresponding execution blocks. it checks
/// that the execution block hashes match the execution payloads in the beacon blocks and that the
/// that the tree hash root of the beacon blocks matches the historical summary block summary root for the era.
pub struct EthereumPostCapellaValidator {
    pub historical_summaries: EthereumBlockSummaryRoots,
}

impl EthereumPostCapellaValidator {
    /// creates a new ethereum post-capella validator.
    pub fn new(historical_summaries: EthereumBlockSummaryRoots) -> Self {
        Self {
            historical_summaries,
        }
    }

    /// validates the era using the post-capella historical summaries.
    ///
    /// input: (execution_block_hashes, beacon_blocks). execution_block_hashes is a vector of
    /// optional execution block hashes, it is optional because not all beacon blocks have an
    /// execution payload. beacon_blocks is a vector of beacon blocks for the era. it is expected
    /// that the execution_block_hash correspond one-to-one with the beacon_blocks.
    pub fn validate_era(
        &self,
        input: (Vec<Option<H256>>, Vec<BeaconBlock<MainnetEthSpec>>),
    ) -> Result<(), EthereumPostCapellaError> {
        self.historical_summaries.validate_era(input)
    }
}

impl EraValidationContext for EthereumBlockSummaryRoots {
    type EraInput = (Vec<Option<H256>>, Vec<BeaconBlock<MainnetEthSpec>>);
    type Error = EthereumPostCapellaError;

    fn validate_era(&self, input: Self::EraInput) -> Result<(), Self::Error> {
        let exec_hashes = input.0;
        let blocks = input.1;

        if blocks.len() != exec_hashes.len() {
            return Err(EthereumPosEraError::MismatchedBlockCount.into());
        }

        for (block, expected_exec_hash) in blocks.iter().zip(exec_hashes.iter()) {
            // Check that the execution block hash matches the expected hash from the beacon block
            // execution payload, if there is one.
            match get_execution_payload_block_hash(block) {
                Some(execution_block_hash) => {
                    // Compare the block hash from the execution payload to the provided hash.
                    let actual_hash = Some(execution_block_hash);
                    if Some(actual_hash) != Some(*expected_exec_hash) {
                        return Err(EthereumPosEraError::ExecutionBlockHashMismatch {
                            expected: *expected_exec_hash,
                            actual: actual_hash,
                        }
                        .into());
                    }
                }
                None => {
                    // If there's no execution payload, make sure no hash was provided.
                    if expected_exec_hash.is_some() {
                        return Err(EthereumPosEraError::ExecutionBlockHashMismatch {
                            expected: None,
                            actual: *expected_exec_hash,
                        }
                        .into());
                    }
                }
            }
        }

        // Get era number from the slot of the first block: era = slot / MAX_EPOCH_SIZE. Return an error if
        // not an even multiple of MAX_EPOCH_SIZE.
        let slot = SlotNumber(blocks[0].slot().into());
        let era: EraNumber = slot.into();
        if slot % MAX_EPOCH_SIZE as u64 != 0 {
            return Err(EthereumPosEraError::InvalidEraStart(slot).into());
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

        // We subract CAPELLA_FORK_EPOCH from the era number to get the index in the historical
        // summaries
        let true_root = {
            let era: u64 = era.into();

            if era < CAPELLA_FORK_EPOCH {
                return Err(EthereumPosEraError::InvalidEraStart(SlotNumber(
                    era * (MAX_EPOCH_SIZE as u64),
                ))
                .into());
            }

            let era_idx = (era - CAPELLA_FORK_EPOCH) as usize;
            if era_idx >= self.0.len() {
                return Err(EthereumPosEraError::EraOutOfBounds {
                    era: era.into(),
                    max_era: EraNumber::from(
                        (self.0.len() + CAPELLA_FORK_EPOCH as usize - 1) as u64,
                    ),
                }
                .into());
            }
            self.0[era_idx]
        };

        if beacon_block_roots_tree_hash_root != FixedBytes::<32>::from(true_root.0) {
            return Err(EthereumPosEraError::InvalidBlockSummaryRoot {
                era,
                expected: true_root,
                actual: beacon_block_roots_tree_hash_root.0.into(),
            }
            .into());
        }

        Ok(())
    }
}
