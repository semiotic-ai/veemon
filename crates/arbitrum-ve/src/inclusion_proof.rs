// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::errors::ArbitrumValidateError;

use alloy_primitives::B256;
use ethportal_api::Header;

/// Off-chain inclusion proof
#[derive(Debug, Clone)]
pub struct OffchainInclusionProof {
    /// The Arbitrum block header to prove
    pub target_header: Header,

    /// The start block hash from an Arbitrum RBlock
    pub start_block_hash: B256,

    /// The end block hash from Arbitrum RBlock
    pub end_block_hash: B256,

    /// The blockheader sequence containing the block
    pub block_header_sequence: Vec<Header>,
}

pub fn generate_offchain_inclusion_proof(
    target_header: Header,
    start_block_hash: B256,
    end_block_hash: B256,
    block_header_sequence: Vec<Header>,
) -> OffchainInclusionProof {
    OffchainInclusionProof {
        target_header,
        start_block_hash,
        end_block_hash,
        block_header_sequence,
    }
}

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

    // Confirm that the start_block_hash is the hash of the first block in the
    // block_header_sequence
    if proof
        .block_header_sequence
        .first()
        .map(|header| header.hash())
        != Some(proof.start_block_hash)
    {
        return Err(ArbitrumValidateError::OffchainInclusionProofVerificationFailure);
    }

    // Confirm that the end_block_hash is the hash of the last block in the block_header_sequence
    if proof
        .block_header_sequence
        .last()
        .map(|header| header.hash())
        != Some(proof.end_block_hash)
    {
        return Err(ArbitrumValidateError::OffchainInclusionProofVerificationFailure);
    }

    // Confirm that the block_header_sequence is correct by confirming that the parent hash for the
    // current header is the hash of the previous header
    for i in 1..proof.block_header_sequence.len() {
        if proof.block_header_sequence[i].parent_hash != proof.block_header_sequence[i - 1].hash() {
            return Err(ArbitrumValidateError::OffchainInclusionProofVerificationFailure);
        }
    }

    Ok(())
}
