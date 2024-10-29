use crate::{epoch::MAX_EPOCH_SIZE, errors::EraValidateError, Epoch};

use ethportal_api::{
    types::execution::{
        accumulator::EpochAccumulator,
        header_with_proof::{BlockHeaderProof, HeaderWithProof, PreMergeAccumulatorProof},
    },
    Header,
};
use tree_hash::Hash256;
use trin_validation::{
    accumulator::PreMergeAccumulator, header_validator::HeaderValidator,
    historical_roots_acc::HistoricalRootsAccumulator,
};

#[derive(Clone)]
pub struct InclusionProof {
    block_number: u64,
    proof: [Hash256; 15],
}

impl InclusionProof {
    pub fn with_header(self, header: Header) -> Result<ProovableHeader, EraValidateError> {
        if self.block_number != header.number {
            Err(EraValidateError::HeaderMismatch {
                expected_number: self.block_number,
                block_number: header.number,
            })
        } else {
            Ok(ProovableHeader {
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

/// generates an inclusion proof over headers, given blocks between `start_block` and `end_block`
///
/// # Arguments
///
/// * `ext_headers`-  A mutable [`Vec<ExtHeaderRecord>`]. The Vector can be any size, however, it must be in chunks of 8192 blocks to work properly
///   to function without error
/// * `start_block` -  The starting point of blocks that are to be included in the proofs. This interval is inclusive.
/// * `end_epoch` -  The ending point of blocks that are to be included in the proofs. This interval is inclusive.
pub fn generate_inclusion_proofs(
    epochs: Vec<Epoch>,
    headers_to_prove: Vec<Header>,
) -> Result<Vec<InclusionProof>, EraValidateError> {
    // We need to load blocks from an entire epoch to be able to generate inclusion proofs
    // First compute epoch accumulators and the Merkle tree for all the epochs of interest
    let mut inclusion_proof_vec: Vec<InclusionProof> = Vec::new();
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

/// verifies an inclusion proof generate by [`generate_inclusion_proof`]
///
/// * `blocks`-  A [`Vec<Block>`]. The blocks included in the inclusion proof interval, set in `start_block` and `end_block` of [`generate_inclusion_proof`]
/// * `pre_merge_accumulator_file`- An instance of [`PreMergeAccumulator`] which is a file that maintains a record of historical epoch
///   it is used to verify canonical-ness of headers accumulated from the `blocks`
/// * `inclusion_proof` -  The inclusion proof generated from [`generate_inclusion_proof`].
pub fn verify_inclusion_proofs(
    pre_merge_accumulator_file: Option<PreMergeAccumulator>,
    header_proofs: Vec<ProovableHeader>,
) -> Result<(), EraValidateError> {
    let pre_merge_acc = pre_merge_accumulator_file.unwrap_or_default();
    let header_validator = HeaderValidator {
        pre_merge_acc,
        historical_roots_acc: HistoricalRootsAccumulator::default(),
    };

    for proovable_header in header_proofs {
        verify_inclusion_proof(
            &header_validator,
            proovable_header.header,
            proovable_header.proof,
        )?;
    }

    Ok(())
}

pub struct ProovableHeader {
    header: Header,
    proof: InclusionProof,
}

pub fn verify_inclusion_proof(
    header_validator: &HeaderValidator,
    header: Header,
    proof: InclusionProof,
) -> Result<(), EraValidateError> {
    let proof = BlockHeaderProof::PreMergeAccumulatorProof(proof.into());

    let hwp = HeaderWithProof { header, proof };

    header_validator
        .validate_header_with_proof(&hwp)
        .map_err(|_| EraValidateError::ProofValidationFailure)
}
