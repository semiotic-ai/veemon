// Copyright (c) 2021-2025 Trin Contributors
// SPDX-License-Identifier: MIT

use alloy_primitives::B256;
use anyhow::anyhow;
use ethportal_api::{
    consensus::historical_summaries::HistoricalSummaries,
    types::execution::header_with_proof_new::{
        BlockHeaderProof, BlockProofHistoricalRoots, BlockProofHistoricalSummaries, HeaderWithProof,
    },
    Header,
};

use crate::{
    constants::{
        CAPELLA_FORK_EPOCH, EPOCH_SIZE, MERGE_BLOCK_NUMBER, SHANGHAI_BLOCK_NUMBER, SLOTS_PER_EPOCH,
    },
    historical_roots::HistoricalRootsAccumulator,
    merkle::proof::verify_merkle_proof,
    PreMergeAccumulator,
};

fn calculate_generalized_index(header: &Header) -> u64 {
    // Calculate generalized index for header
    // https://github.com/ethereum/consensus-specs/blob/v0.11.1/ssz/merkle-proofs.md#generalized-merkle-tree-index
    let hr_index = header.number % EPOCH_SIZE;
    (EPOCH_SIZE * 2 * 2) + (hr_index * 2)
}

/// HeaderValidator is responsible for validating pre-merge and post-merge headers with their
/// respective proofs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HeaderValidator {
    /// Pre-merge accumulator used to validate pre-merge headers.
    pub pre_merge_acc: PreMergeAccumulator,
    /// Historical roots accumulator used to validate post-merge/pre-Capella headers.
    pub historical_roots_acc: HistoricalRootsAccumulator,
}

impl HeaderValidator {
    pub fn new() -> Self {
        let pre_merge_acc = PreMergeAccumulator::default();
        let historical_roots_acc = HistoricalRootsAccumulator::default();

        Self {
            pre_merge_acc,
            historical_roots_acc,
        }
    }

    pub fn validate_header_with_proof(&self, hwp: &HeaderWithProof) -> anyhow::Result<()> {
        match &hwp.proof {
            BlockHeaderProof::HistoricalHashes(proof) => {
                if hwp.header.number > MERGE_BLOCK_NUMBER {
                    return Err(anyhow!("Invalid proof type found for post-merge header."));
                }
                // Look up historical epoch hash for header from pre-merge accumulator
                let gen_index = calculate_generalized_index(&hwp.header);
                let epoch_index =
                    self.pre_merge_acc.get_epoch_index_of_header(&hwp.header) as usize;
                let epoch_hash = self.pre_merge_acc.historical_epochs[epoch_index];

                match verify_merkle_proof(
                    hwp.header.hash(),
                    proof,
                    15,
                    gen_index as usize,
                    epoch_hash,
                ) {
                    true => Ok(()),
                    false => Err(anyhow!(
                        "Merkle proof validation failed for pre-merge header"
                    )),
                }
            }
            BlockHeaderProof::HistoricalRoots(proof) => self.verify_post_merge_pre_capella_header(
                hwp.header.number,
                hwp.header.hash(),
                proof,
            ),
            BlockHeaderProof::HistoricalSummaries(_) => {
                if hwp.header.number < SHANGHAI_BLOCK_NUMBER {
                    return Err(anyhow!(
                        "Invalid BlockProofHistoricalSummaries found for pre-Shanghai header."
                    ));
                }
                // TODO: Validation for post-Capella headers is not implemented
                Ok(())
            }
        }
    }

    /// A method to verify the chain of proofs for post-merge/pre-Capella execution headers.
    fn verify_post_merge_pre_capella_header(
        &self,
        block_number: u64,
        header_hash: B256,
        proof: &BlockProofHistoricalRoots,
    ) -> anyhow::Result<()> {
        if block_number <= MERGE_BLOCK_NUMBER {
            return Err(anyhow!(
                "Invalid HistoricalRootsBlockProof found for pre-merge header."
            ));
        }
        if block_number >= SHANGHAI_BLOCK_NUMBER {
            return Err(anyhow!(
                "Invalid HistoricalRootsBlockProof found for post-Shanghai header."
            ));
        }

        // Verify the chain of proofs for post-merge/pre-capella block header
        Self::verify_beacon_block_proof(
            header_hash,
            &proof.execution_block_proof,
            proof.beacon_block_root,
        )?;

        let block_root_index = proof.slot % EPOCH_SIZE;
        let gen_index = 2 * EPOCH_SIZE + block_root_index;
        let historical_root_index = proof.slot / EPOCH_SIZE;
        let historical_root =
            self.historical_roots_acc.historical_roots[historical_root_index as usize];

        if !verify_merkle_proof(
            proof.beacon_block_root,
            &proof.beacon_block_proof,
            14,
            gen_index as usize,
            historical_root,
        ) {
            return Err(anyhow!(
                "Merkle proof validation failed for HistoricalRootsProof"
            ));
        }

        Ok(())
    }

    /// A method to verify the chain of proofs for post-Capella execution headers.
    #[allow(dead_code)] // TODO: Remove this when used
    fn verify_post_capella_header(
        &self,
        block_number: u64,
        header_hash: B256,
        proof: &BlockProofHistoricalSummaries,
        historical_summaries: HistoricalSummaries,
    ) -> anyhow::Result<()> {
        if block_number < SHANGHAI_BLOCK_NUMBER {
            return Err(anyhow!(
                "Invalid HistoricalSummariesBlockProof found for pre-Shanghai header."
            ));
        }

        // Verify the chain of proofs for post-merge/pre-capella block header
        Self::verify_beacon_block_proof(
            header_hash,
            &proof.execution_block_proof,
            proof.beacon_block_root,
        )?;

        let block_root_index = proof.slot % EPOCH_SIZE;
        let gen_index = EPOCH_SIZE + block_root_index;
        let historical_summary_index =
            (proof.slot - CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH) / EPOCH_SIZE;
        let historical_summary =
            historical_summaries[historical_summary_index as usize].block_summary_root;

        if !verify_merkle_proof(
            proof.beacon_block_root,
            &proof.beacon_block_proof,
            13,
            gen_index as usize,
            historical_summary,
        ) {
            return Err(anyhow!(
                "Merkle proof validation failed for HistoricalSummariesProof"
            ));
        }

        Ok(())
    }

    /// Verify that the execution block header is included in the beacon block
    fn verify_beacon_block_proof(
        header_hash: B256,
        block_body_proof: &[B256],
        block_body_root: B256,
    ) -> anyhow::Result<()> {
        // BeaconBlock level:
        // - 8 as there are 5 fields
        // - 4 as index (pos) of field is 4
        // let gen_index_top_level = (1 * 1 * 8 + 4)
        // BeaconBlockBody level:
        // - 16 as there are 10 fields
        // - 9 as index (pos) of field is 9
        // let gen_index_mid_level = (gen_index_top_level * 1 * 16 + 9)
        // ExecutionPayload level:
        // - 16 as there are 14 fields
        // - 12 as pos of field is 12
        // let gen_index = (gen_index_mid_level * 1 * 16 + 12) = 3228
        let gen_index = 3228;

        if !verify_merkle_proof(
            header_hash,
            block_body_proof,
            block_body_proof.len(),
            gen_index,
            block_body_root,
        ) {
            return Err(anyhow!(
                "Merkle proof validation failed for BeaconBlockProof"
            ));
        }
        Ok(())
    }
}
