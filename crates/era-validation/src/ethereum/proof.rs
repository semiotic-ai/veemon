// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{error::AuthenticationError, ethereum::types::MAX_EPOCH_SIZE, Epoch};

use alloy_consensus::Header;
use alloy_primitives::FixedBytes;
use ethportal_api::consensus::historical_summaries::HistoricalSummaries;
use ethportal_api::types::execution::{
    accumulator::EpochAccumulator,
    header_with_proof::{
        BlockHeaderProof, BlockProofHistoricalHashesAccumulator,
        HeaderWithProof as PortalHeaderWithProof,
    },
};
use validation::{
    header_validator::HeaderValidator, historical_roots::HistoricalRootsAccumulator,
    PreMergeAccumulator,
};

const PROOF_SIZE: usize = 15;

/// A proof that contains the block number
#[derive(Clone)]
pub struct InclusionProof {
    block_number: u64,
    proof: [FixedBytes<32>; PROOF_SIZE],
}

impl InclusionProof {
    /// Takes a header and turns the proof into a provable header
    pub fn with_header(self, header: Header) -> Result<HeaderWithProof, AuthenticationError> {
        if self.block_number != header.number {
            Err(AuthenticationError::HeaderMismatch {
                expected_number: self.block_number,
                block_number: header.number,
            })
        } else {
            Ok(HeaderWithProof {
                proof: self,
                header,
            })
        }
    }
}

/// Generates inclusion proofs for headers, given a list epochs that contains
/// the headers to be proven
///
/// This function creates Merkle inclusion proofs for specified headers by looking them up
/// within the provided epochs. Each proof demonstrates that a header's block hash is correctly
/// included in its epoch's accumulator, which can then be verified against the historical
/// PreMergeAccumulator.
///
/// # Arguments
///
/// * `epochs` - A list of epochs [`Vec<Epoch>`] containing the block headers. Each epoch
///   represents 8192 blocks (ERA size).
/// * `headers_to_prove` - A list of headers [`Vec<Header>`] for which to generate inclusion proofs.
///   These headers must exist within the provided epochs.
///
/// # Returns
///
/// * `Ok(Vec<InclusionProof>)` - A vector of inclusion proofs, one for each header
/// * `Err(AuthenticationError)` - If a header's epoch is not found in the provided list, or if
///   proof generation fails
///
/// # Example
///
/// This example demonstrates generating inclusion proofs for multiple blocks across different
/// epochs, which is useful when you need to verify specific blocks from a larger dataset.
///
/// ```ignore
/// use std::{fs::File, io::BufReader};
/// use decoder::{read_blocks_from_reader, AnyBlock, Compression};
/// use era_validation::{Epoch, ExtHeaderRecord, ethereum::generate_inclusion_proofs};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // read headers from multiple eras (epochs 0 and 1)
/// let mut all_headers: Vec<ExtHeaderRecord> = Vec::new();
///
/// // load epoch 0 (blocks 0-8191)
/// for block_num in (0..=8100).step_by(100) {
///     let path = format!("path/to/dbin/epoch0/{:010}.dbin", block_num);
///     let reader = BufReader::new(File::open(path)?);
///     let blocks = read_blocks_from_reader(reader, Compression::None)?;
///     all_headers.extend(
///         blocks.iter().filter_map(|b| {
///             if let AnyBlock::Evm(eth) = b {
///                 ExtHeaderRecord::try_from(eth).ok()
///             } else {
///                 None
///             }
///         })
///     );
/// }
///
/// // load epoch 1 (blocks 8192-16383)
/// for block_num in (8192..=16300).step_by(100) {
///     let path = format!("path/to/dbin/epoch1/{:010}.dbin", block_num);
///     let reader = BufReader::new(File::open(path)?);
///     let blocks = read_blocks_from_reader(reader, Compression::None)?;
///     all_headers.extend(
///         blocks.iter().filter_map(|b| {
///             if let AnyBlock::Evm(eth) = b {
///                 ExtHeaderRecord::try_from(eth).ok()
///             } else {
///                 None
///             }
///         })
///     );
/// }
///
/// // extract full headers before creating epochs
/// let headers_to_prove: Vec<_> = all_headers
///     .iter()
///     .filter(|h| [100, 1000, 8242].contains(&h.block_number))
///     .filter_map(|ext| ext.full_header.as_ref().cloned())
///     .collect();
///
/// // separate headers by epoch
/// let (epoch0_headers, epoch1_headers): (Vec<_>, Vec<_>) = all_headers
///     .into_iter()
///     .partition(|h| h.block_number < 8192);
///
/// // create epochs
/// let epoch0: Epoch = epoch0_headers.try_into()?;
/// let epoch1: Epoch = epoch1_headers.try_into()?;
/// let epochs = vec![epoch0, epoch1];
///
/// // generate proofs for all selected headers
/// let proofs = generate_inclusion_proofs(epochs, headers_to_prove.clone())?;
///
/// // verify we got one proof per header
/// assert_eq!(proofs.len(), headers_to_prove.len());
///
/// // combine proofs with headers for verification
/// let provable_headers: Vec<_> = headers_to_prove
///     .into_iter()
///     .zip(proofs)
///     .map(|(header, proof)| proof.with_header(header))
///     .collect::<Result<Vec<_>, _>>()?;
///
/// println!("generated {} inclusion proofs", provable_headers.len());
/// # Ok(())
/// # }
/// ```
///
/// # See Also
///
/// - [`generate_inclusion_proof`] for generating a proof for a single header
/// - [`verify_inclusion_proofs`] for verifying the generated proofs
pub fn generate_inclusion_proofs(
    epochs: Vec<Epoch>,
    headers_to_prove: Vec<Header>,
) -> Result<Vec<InclusionProof>, AuthenticationError> {
    let mut inclusion_proof_vec: Vec<InclusionProof> = Vec::with_capacity(headers_to_prove.len());
    let epoch_list: Vec<_> = epochs.iter().map(|epoch| epoch.number() as u64).collect();
    let accumulators: Vec<_> = epochs
        .into_iter()
        .map(|epoch| (epoch.number(), EpochAccumulator::from(epoch)))
        .collect();

    for header in headers_to_prove {
        let block_epoch = header.number / MAX_EPOCH_SIZE as u64;

        let accumulator = accumulators
            .iter()
            .find(|epoch| epoch.0 as u64 == block_epoch)
            .map(|epoch| &epoch.1)
            .ok_or(AuthenticationError::EpochNotFoundInProvidedList {
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
) -> Result<InclusionProof, AuthenticationError> {
    let block_number = header.number;
    let block_epoch = block_number / MAX_EPOCH_SIZE as u64;
    if block_epoch != epoch.number() as u64 {
        return Err(AuthenticationError::EpochNotMatchForHeader {
            epoch_number: epoch.number() as u64,
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
) -> Result<InclusionProof, AuthenticationError> {
    PreMergeAccumulator::construct_proof(header, epoch_accumulator)
        .map(|proof| {
            // convert BlockProofHistoricalHashesAccumulator to [FixedBytes<32>; 15]
            // the proof is a FixedVector<B256, U15>, so we can iterate over it
            let proof_array: [FixedBytes<32>; PROOF_SIZE] = proof
                .iter()
                .map(|b| FixedBytes::from_slice(b.as_slice()))
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| AuthenticationError::ProofGenerationFailure)?;

            Ok(InclusionProof {
                block_number: header.number,
                proof: proof_array,
            })
        })
        .map_err(|_| AuthenticationError::ProofGenerationFailure)?
}

/// Verifies a list of provable headers
///
/// This function validates that execution layer block headers are part of the canonical
/// Ethereum chain by verifying inclusion proofs. Currently, this function generates proofs
/// compatible with pre-merge block validation using `BlockHeaderProof::HistoricalHashes`.
///
/// **Note**: For post-Capella validation (blocks ≥17,034,870), use [`HeaderValidator`](validation::header_validator::HeaderValidator)
/// directly with `BlockProofHistoricalSummariesCapella` or `BlockProofHistoricalSummariesDeneb` proofs.
///
/// # Arguments
///
/// * `pre_merge_accumulator_file` - An optional [`PreMergeAccumulator`] containing
///   historical epoch accumulator roots. Pass `None` to use the default embedded accumulator.
/// * `header_proofs` - A [`Vec<HeaderWithProof>`] containing headers and their inclusion proofs
/// * `historical_summaries` - Reserved for future post-Capella support
///
/// # Returns
///
/// * `Ok(())` if all header proofs verify successfully
/// * `Err(AuthenticationError)` - If any proof fails validation
///
/// # Example: Pre-Merge Era Validation
///
/// This example shows the complete workflow for verifying pre-merge blocks (blocks 0-15,537,394):
/// reading blocks from .dbin files, generating inclusion proofs, and verifying them.
///
/// **Note**: Test .dbin files can be obtained from:
/// - [ve-assets repository](https://github.com/semiotic-ai/ve-assets) for sample pre-merge blocks
/// - streamingfast firehose extraction using a provider like pinax
///
/// ```ignore
/// use std::{fs::File, io::BufReader};
/// use decoder::{read_blocks_from_reader, AnyBlock, Compression};
/// use era_validation::{
///     Epoch, ExtHeaderRecord, ethereum::{generate_inclusion_proofs, verify_inclusion_proofs},
/// };
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // step 1: read blocks from .dbin files (pre-merge era blocks 0-8199)
/// let mut headers: Vec<ExtHeaderRecord> = Vec::new();
/// for block_num in (0..=8200).step_by(100) {
///     let path = format!("path/to/dbin/{:010}.dbin", block_num);
///     let reader = BufReader::new(File::open(path)?);
///     let blocks = read_blocks_from_reader(reader, Compression::None)?;
///
///     headers.extend(
///         blocks.iter().filter_map(|block| {
///             if let AnyBlock::Evm(eth_block) = block {
///                 ExtHeaderRecord::try_from(eth_block).ok()
///             } else {
///                 None
///             }
///         })
///     );
/// }
///
/// // step 2: select specific blocks to prove (e.g., blocks 100-200)
/// let headers_to_prove: Vec<_> = headers[100..200]
///     .iter()
///     .map(|ext| ext.full_header.as_ref().unwrap().clone())
///     .collect();
///
/// // step 3: create epoch from all headers in the era (8192 blocks)
/// let epoch: Epoch = headers.try_into()?;
///
/// // step 4: generate inclusion proofs for the selected headers
/// let inclusion_proofs = generate_inclusion_proofs(
///     vec![epoch],
///     headers_to_prove.clone()
/// )?;
///
/// // step 5: combine headers with their proofs
/// let provable_headers = headers_to_prove
///     .into_iter()
///     .zip(inclusion_proofs)
///     .map(|(header, proof)| proof.with_header(header))
///     .collect::<Result<Vec<_>, _>>()?;
///
/// // step 6: verify inclusion proofs using PreMergeAccumulator
/// // the default accumulator contains historical epoch roots from ethereum portal network
/// verify_inclusion_proofs(None, provable_headers, None)?;
///
/// println!("pre-merge blocks verified successfully");
/// # Ok(())
/// # }
/// ```
///
/// # See Also
///
/// For post-Capella validation examples, see:
/// - [`HeaderValidator::verify_post_capella_header`](validation::header_validator::HeaderValidator::verify_post_capella_header)
///   for Capella and Deneb era validation
/// - [`PostCapellaProof`](validation::header_validator::PostCapellaProof) for era-specific proof structures
/// - [`generate_inclusion_proofs`] for creating proofs from epochs
pub fn verify_inclusion_proofs(
    pre_merge_accumulator_file: Option<PreMergeAccumulator>,
    header_proofs: Vec<HeaderWithProof>,
    historical_summaries: Option<HistoricalSummaries>,
) -> Result<(), AuthenticationError> {
    let pre_merge_acc = pre_merge_accumulator_file.unwrap_or_default();
    let header_validator = HeaderValidator {
        pre_merge_acc,
        historical_roots_acc: HistoricalRootsAccumulator::default(),
        historical_summaries,
    };

    for provable_header in header_proofs {
        verify_inclusion_proof(&header_validator, provable_header)?;
    }

    Ok(())
}

/// A header with an inclusion proof attached
pub struct HeaderWithProof {
    header: Header,
    proof: InclusionProof,
}

/// Verifies if a proof is contained in the header validator
pub fn verify_inclusion_proof(
    header_validator: &HeaderValidator,
    provable_header: HeaderWithProof,
) -> Result<(), AuthenticationError> {
    // Convert [FixedBytes<32>; 15] to Vec<B256> for BlockProofHistoricalHashesAccumulator
    let proof_vec: Vec<alloy_primitives::B256> = provable_header
        .proof
        .proof
        .iter()
        .map(|fixed_bytes| alloy_primitives::B256::from_slice(fixed_bytes.as_slice()))
        .collect();

    let block_proof = BlockProofHistoricalHashesAccumulator::new(proof_vec)
        .map_err(|_| AuthenticationError::ProofValidationFailure)?;

    let proof = BlockHeaderProof::HistoricalHashes(block_proof);

    let hwp = PortalHeaderWithProof {
        header: provable_header.header,
        proof,
    };

    header_validator
        .validate_header_with_proof(&hwp)
        .map_err(|_| AuthenticationError::ProofValidationFailure)
}
