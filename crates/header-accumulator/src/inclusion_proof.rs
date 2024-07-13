use crate::{
    errors::EraValidateError,
    types::ExtHeaderRecord,
    utils::{compute_epoch_accumulator, header_from_block, MAX_EPOCH_SIZE},
};

use alloy_primitives::FixedBytes;
use ethportal_api::{
    types::execution::{
        accumulator::HeaderRecord,
        header_with_proof::{BlockHeaderProof, HeaderWithProof, PreMergeAccumulatorProof},
    },
    Header,
};
use sf_protos::ethereum::r#type::v2::Block;
use trin_validation::{
    accumulator::PreMergeAccumulator, header_validator::HeaderValidator,
    historical_roots_acc::HistoricalRootsAccumulator,
};

/// generates an inclusion proof over headers, given blocks between `start_block` and `end_block`
///
/// # Arguments
///
/// * `ext_headers`-  A mutable [`Vec<ExtHeaderRecord>`]. The Vector can be any size, however, it must be in chunks of 8192 blocks to work properly
/// to function without error
/// * `start_block` -  The starting point of blocks that are to be included in the proofs. This interval is inclusive.
/// * `end_epoch` -  The ending point of blocks that are to be included in the proofs. This interval is inclusive.
pub fn generate_inclusion_proof(
    mut ext_headers: Vec<ExtHeaderRecord>,
    start_block: u64,
    end_block: u64,
) -> Result<Vec<[FixedBytes<32>; 15]>, EraValidateError> {
    // Compute the epoch accumulator for the blocks
    // The epochs start on a multiple of 8192 blocks, so we need to round down to the nearest 8192
    let epoch_start = start_block / MAX_EPOCH_SIZE as u64;

    // The epochs end on a multiple of 8192 blocks, so we need to round up to the nearest 8192
    let epoch_end = ((end_block as f32) / MAX_EPOCH_SIZE as f32).ceil() as u64;

    // We need to load blocks from an entire epoch to be able to generate inclusion proofs
    // First compute epoch accumulators and the Merkle tree for all the epochs of interest
    let mut epoch_accumulators = Vec::new();
    let mut inclusion_proof_vec: Vec<[FixedBytes<32>; 15]> = Vec::new();
    let mut headers: Vec<Header> = Vec::new();

    for _ in epoch_start..epoch_end {
        let epoch_headers: Vec<ExtHeaderRecord> = ext_headers.drain(0..MAX_EPOCH_SIZE).collect();
        let header_records: Vec<HeaderRecord> = epoch_headers.iter().map(Into::into).collect();
        let tmp_headers: Vec<Header> = epoch_headers.into_iter().map(ExtHeaderRecord::try_into).collect::<Result<_, _>>()?;
        headers.extend(tmp_headers);
        epoch_accumulators.push(compute_epoch_accumulator(&header_records)?);
    }

    for block_idx in start_block..=end_block {
        let epoch = block_idx / MAX_EPOCH_SIZE as u64;
        let epoch_acc = epoch_accumulators[epoch as usize].clone();
        let header = headers[block_idx as usize].clone();
        inclusion_proof_vec.push(
            PreMergeAccumulator::construct_proof(&header, &epoch_acc)
                .map_err(|_| EraValidateError::ProofGenerationFailure)?,
        );
    }

    Ok(inclusion_proof_vec)
}

/// verifies an inclusion proof generate by [`generate_inclusion_proof`]
///
/// * `blocks`-  A [`Vec<Block>`]. The blocks included in the inclusion proof interval, set in `start_block` and `end_block` of [`generate_inclusion_proof`]
/// * `pre_merge_accumulator_file`- An instance of [`PreMergeAccumulator`] which is a file that maintains a record of historical epoch
/// it is used to verify canonical-ness of headers accumulated from the `blocks`
/// * `inclusion_proof` -  The inclusion proof generated from [`generate_inclusion_proof`].
pub fn verify_inclusion_proof(
    blocks: Vec<Block>,
    pre_merge_accumulator_file: Option<PreMergeAccumulator>,
    inclusion_proof: Vec<[FixedBytes<32>; 15]>,
) -> Result<(), EraValidateError> {
    let pre_merge_acc = pre_merge_accumulator_file.unwrap_or_default();

    let header_validator = HeaderValidator {
        pre_merge_acc,
        historical_roots_acc: HistoricalRootsAccumulator::default(),
    };

    for (block_idx, _) in blocks.iter().enumerate() {
        let bhp = BlockHeaderProof::PreMergeAccumulatorProof(PreMergeAccumulatorProof {
            proof: inclusion_proof[block_idx],
        });

        let hwp = HeaderWithProof {
            header: header_from_block(&blocks[block_idx].clone())?,
            proof: bhp,
        };

        header_validator
            .validate_header_with_proof(&hwp)
            .map_err(|_| EraValidateError::ProofValidationFailure)?;
    }

    Ok(())
}
