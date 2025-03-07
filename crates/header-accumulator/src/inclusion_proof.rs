// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{epoch::MAX_EPOCH_SIZE, errors::EraValidateError, Epoch};

use ethportal_api::types::execution::header_with_proof_new::BlockHeaderProof;
use ethportal_api::types::execution::{
    accumulator::EpochAccumulator, header::Header, header_with_proof_new::HeaderWithProof,
};

use ethportal_api::MERGE_TIMESTAMP;
use validation::constants::{CAPELLA_BLOCK_NUMBER, MERGE_BLOCK_NUMBER};
use validation::{
    header_validator::HeaderValidator, historical_roots::HistoricalRootsAccumulator,
    PreMergeAccumulator,
};

/// A proof that contains the block number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InclusionProof {
    /// given block number
    pub block_number: u64,
    /// A proof of the given BlockHeader
    pub proof: BlockHeaderProof,
}

impl InclusionProof {
    /// creates a proof for the header, unites both proof and header into InclusionProof sturct
    pub fn with_header(self, header: Header) -> Result<HeaderWithProof, EraValidateError> {
        if self.block_number != header.number {
            return Err(EraValidateError::HeaderMismatch {
                expected_number: self.block_number,
                block_number: header.number,
            });
        }

        // Ensure that the proof actually corresponds to the correct historical accumulator
        let proof_era_valid = match &self.proof {
            BlockHeaderProof::HistoricalHashes(_) => header.number < MERGE_BLOCK_NUMBER,
            BlockHeaderProof::HistoricalRoots(_) => {
                (MERGE_BLOCK_NUMBER..CAPELLA_BLOCK_NUMBER).contains(&header.number)
            }
            BlockHeaderProof::HistoricalSummaries(_) => header.number >= CAPELLA_BLOCK_NUMBER,
        };

        if !proof_era_valid {
            return Err(EraValidateError::InvalidProofEra {
                timestamp: self.block_number, // add code here
            });
        }

        Ok(HeaderWithProof {
            proof: self.proof,
            header,
        })
    }
}

/// Generates inclusion proofs for headers, given a list epochs that contains
/// the headers to be proven
///
/// # Arguments
///
/// * `epochs`-  A list of epochs [`Vec<Epoch>`].
/// * `headers_to_prove` - A list of headers [`Vec<Header>`]
pub fn generate_inclusion_proofs(
    epochs: Vec<Epoch>,
    headers_to_prove: Vec<Header>,
) -> Result<Vec<InclusionProof>, EraValidateError> {
    let mut inclusion_proof_vec: Vec<InclusionProof> = Vec::with_capacity(headers_to_prove.len());
    let epoch_list: Vec<_> = epochs.iter().map(|epoch| epoch.number()).collect();
    let accumulators: Vec<_> = epochs
        .into_iter()
        .map(|epoch| (epoch.number(), EpochAccumulator::from(epoch)))
        .collect();

    for header in headers_to_prove {
        let block_epoch = (header.number / MAX_EPOCH_SIZE as u64) as usize;

        let accumulator = accumulators
            .iter()
            .find(|epoch| epoch.0 == block_epoch)
            .map(|epoch| &epoch.1)
            .ok_or(EraValidateError::EpochNotFoundInProvidedList {
                block_epoch,
                epoch_list: epoch_list.clone(),
            })?;

        inclusion_proof_vec.push(do_generate_inclusion_proof(&header, accumulator)?);
    }

    Ok(inclusion_proof_vec)
}

/// Generates an inclusion proof for the header, given the epoch that contains
/// the header to be proven
///
/// Returns an error if the header is not inside the epoch.
///
/// # Arguments
///
/// * `header`- Header to be proven
/// * `epoch` - Epoch in which the header is located
pub fn generate_inclusion_proof(
    header: Header,
    epoch: Epoch,
) -> Result<InclusionProof, EraValidateError> {
    let block_number = header.number;
    let block_epoch = (block_number / MAX_EPOCH_SIZE as u64) as usize;
    if block_epoch != epoch.number() {
        return Err(EraValidateError::EpochNotMatchForHeader {
            epoch_number: epoch.number(),
            block_number,
            block_epoch,
        });
    }

    let epoch_accumulator = EpochAccumulator::from(epoch);
    do_generate_inclusion_proof(&header, &epoch_accumulator)
}

fn do_generate_inclusion_proof(
    header: &Header,
    epoch_accumulator: &EpochAccumulator,
) -> Result<InclusionProof, EraValidateError> {
    if header.timestamp > MERGE_TIMESTAMP {
        return Err(EraValidateError::InvalidProofEra {
            timestamp: header.timestamp,
        });
    }

    let proof = PreMergeAccumulator::construct_proof(header, epoch_accumulator)
        .map_err(|_| EraValidateError::ProofGenerationFailure)?; // Convert anyhow::Error to EraValidateError

    Ok(InclusionProof {
        proof: BlockHeaderProof::HistoricalHashes(proof),
        block_number: header.number,
    })
}

/// Verifies a list of provable headers
///
/// * `pre_merge_accumulator_file`- An optional instance of [`PreMergeAccumulator`]
///     which is a file that maintains a record of historical epoch it is used to
///     verify canonical-ness of headers accumulated from the `blocks`
/// * `header_proofs`-  A [`Vec<HeaderWithProof>`].
pub fn verify_inclusion_proofs(
    pre_merge_accumulator_file: Option<PreMergeAccumulator>,
    header_proofs: Vec<HeaderWithProof>,
) -> Result<(), EraValidateError> {
    let pre_merge_acc = pre_merge_accumulator_file.unwrap_or_default();
    let header_validator = HeaderValidator {
        pre_merge_acc,
        historical_roots_acc: HistoricalRootsAccumulator::default(),
    };

    for provable_header in header_proofs {
        verify_inclusion_proof(&header_validator, provable_header)?;
    }

    Ok(())
}
/// Verifies if a proof is contained in the header validator
pub fn verify_inclusion_proof(
    header_validator: &HeaderValidator,
    provable_header: HeaderWithProof,
) -> Result<(), EraValidateError> {
    header_validator
        .validate_header_with_proof(&provable_header)
        .map_err(|_| EraValidateError::ProofValidationFailure)
}
