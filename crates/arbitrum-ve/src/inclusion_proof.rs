// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::errors::ArbitrumValidateError;

use alloy_consensus::Header;
use alloy_primitives::B256;

/// Off-chain inclusion proof
#[derive(Debug, Clone)]
pub struct OffchainInclusionProof {
    /// The Arbitrum block header to prove
    pub target_header: Header,

    /// The end block hash from the previous Arbitrum RBlock
    pub prev_end_block_hash: B256,

    /// The end block hash from Arbitrum RBlock
    pub end_block_hash: B256,

    /// The blockheader sequence containing the block
    pub block_header_sequence: Vec<Header>,
}

/// The off-chain inclusion proof is relatively trivial. It consists simply of the Arbitrum block
/// header to be verified `target_header`, the end block hash `prev_block_hash` indicated the previous RBlock, the end block hash `end_block_hash` indicated
/// in the same RBlock, and all Arbitrum block headers between the previous end block hash and the current end block hashes
/// `block_header_sequence`.
pub fn generate_offchain_inclusion_proof(
    target_header: Header,
    prev_end_block_hash: B256,
    end_block_hash: B256,
    block_header_sequence: Vec<Header>,
) -> OffchainInclusionProof {
    OffchainInclusionProof {
        target_header,
        prev_end_block_hash,
        end_block_hash,
        block_header_sequence,
    }
}

/// Off-chain proof verification is simple. We simply confirm that the target header is inclueded
/// in the sequence of block headers between the previous end_block and the current end block hashes, then we confirm that
/// the block header hash sequence is correct.
pub fn verify_offchain_inclusion_proof(
    proof: &OffchainInclusionProof,
) -> Result<(), ArbitrumValidateError> {
    // Confirm that the target_header is in the block_header_sequence
    if !proof
        .block_header_sequence
        .iter()
        .any(|header| header == &proof.target_header)
    {
        return Err(ArbitrumValidateError::OffchainInclusionProofVerificationFailure);
    }

    // Confirm that the parent hash of the first header in the sequence is the prev_end_block_hash
    if proof
        .block_header_sequence
        .first()
        .map(|header| header.parent_hash)
        != Some(proof.prev_end_block_hash)
    {
        return Err(ArbitrumValidateError::OffchainInclusionProofVerificationFailure);
    }

    // Confirm that the end_block_hash is the hash of the last block in the block_header_sequence
    if proof
        .block_header_sequence
        .last()
        .map(|header| header.hash_slow())
        != Some(proof.end_block_hash)
    {
        return Err(ArbitrumValidateError::OffchainInclusionProofVerificationFailure);
    }

    // Confirm that the block_header_sequence is correct by confirming that the parent hash for the
    // current header is the hash of the previous header
    for i in 1..proof.block_header_sequence.len() {
        if proof.block_header_sequence[i].parent_hash
            != proof.block_header_sequence[i - 1].hash_slow()
        {
            return Err(ArbitrumValidateError::OffchainInclusionProofVerificationFailure);
        }
    }

    Ok(())
}
