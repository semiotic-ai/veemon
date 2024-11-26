use crate::{epoch::MAX_EPOCH_SIZE, errors::EraValidateError, Epoch};

use ethportal_api::{
    types::execution::{
        accumulator::EpochAccumulator,
        header_with_proof::{
            BlockHeaderProof, HeaderWithProof as PortalHeaderWithProof, PreMergeAccumulatorProof,
        },
    },
    Header,
};
use tree_hash::Hash256;
use trin_validation::{
    accumulator::PreMergeAccumulator, header_validator::HeaderValidator,
    historical_roots_acc::HistoricalRootsAccumulator,
};

const PROOF_SIZE: usize = 15;

/// A proof that contains the block number
#[derive(Clone)]
pub struct InclusionProof {
    block_number: u64,
    proof: [Hash256; PROOF_SIZE],
}

impl InclusionProof {
    /// Takes a header and turns the proof into a provable header
    pub fn with_header(self, header: Header) -> Result<HeaderWithProof, EraValidateError> {
        if self.block_number != header.number {
            Err(EraValidateError::HeaderMismatch {
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

impl From<InclusionProof> for PreMergeAccumulatorProof {
    fn from(value: InclusionProof) -> Self {
        Self { proof: value.proof }
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
    PreMergeAccumulator::construct_proof(header, epoch_accumulator)
        .map(|proof| InclusionProof {
            proof,
            block_number: header.number,
        })
        .map_err(|_| EraValidateError::ProofGenerationFailure)
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

/// A header with an inclusion proof attached
pub struct HeaderWithProof {
    header: Header,
    proof: InclusionProof,
}

/// Verifies if a proof is contained in the header validator
pub fn verify_inclusion_proof(
    header_validator: &HeaderValidator,
    provable_header: HeaderWithProof,
) -> Result<(), EraValidateError> {
    let proof = BlockHeaderProof::PreMergeAccumulatorProof(provable_header.proof.into());

    let hwp = PortalHeaderWithProof {
        header: provable_header.header,
        proof,
    };

    header_validator
        .validate_header_with_proof(&hwp)
        .map_err(|_| EraValidateError::ProofValidationFailure)
}
