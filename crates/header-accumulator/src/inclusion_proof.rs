use crate::{
    errors::EraValidateError,
    utils::{
        compute_epoch_accumulator, decode_header_records, extract_100_blocks, header_from_block,
    },
};
use ethportal_api::{AccumulatorProof, BlockHeaderProof, HeaderWithProof};
use primitive_types::H256;
use trin_validation::accumulator::MasterAccumulator;

// function: generate_inclusion_proof
// inputs: flat_file_directory, master_accumulator_file, start_block, end_block
// outputs: inclusion_proof
pub fn generate_inclusion_proof(
    directory: &String,
    start_block: usize,
    end_block: usize,
) -> Result<Vec<[H256; 15]>, EraValidateError> {
    // Compute the epoch accumulator for the blocks
    // The epochs start on a multiple of 8192 blocks, so we need to round down to the nearest 8192
    let epoch_start = start_block / 8192;

    // The epochs end on a multiple of 8192 blocks, so we need to round up to the nearest 8192
    let epoch_end = ((end_block as f32) / 8192.0).ceil() as usize;

    // We need to load blocks from an entire epoch to be able to generate inclusion proofs
    // First compute epoch accumulators and the Merkle tree for all the epochs of interest
    let mut epoch_accumulators = Vec::new();
    let mut inclusion_proof_vec = Vec::new();
    let mut headers = Vec::new();
    for epoch in epoch_start..epoch_end {
        let start_block = epoch * 8192;
        let end_block = (epoch + 1) * 8192;

        let blocks = extract_100_blocks(directory, start_block, end_block)?;


        let mut blocks_headers = Vec::new();
        for block in blocks.clone() {
            let header = header_from_block(block)?;
            blocks_headers.push(header.clone());

        }
        let header_records = decode_header_records(blocks)?;
        headers.extend(blocks_headers);
        epoch_accumulators.push(compute_epoch_accumulator(&header_records)?);
    }

    for block_idx in start_block..end_block {
        let epoch = block_idx / 8192;
        let epoch_acc = epoch_accumulators[epoch].clone();
        let header = headers[block_idx].clone();
        inclusion_proof_vec.push(
            MasterAccumulator::construct_proof(&header, &epoch_acc)
                .map_err(|_| EraValidateError::ProofGenerationFailure)?,
        );
    }
    Ok(inclusion_proof_vec)
}

// function: verify_inclusion_proof
// inputs: flat_file_directory, block_range, inclusion_proof
// outputs: result
pub fn verify_inclusion_proof(
    directory: &String,
    master_accumulator_file: Option<&String>,
    start_block: usize,
    end_block: usize,
    inclusion_proof: Vec<[H256; 15]>,
) -> Result<(), EraValidateError> {
    let num_blocks = end_block - start_block;

    // Load master accumulator
    let master_accumulator = match master_accumulator_file {
        Some(master_accumulator_file) => {
            MasterAccumulator::try_from_file(master_accumulator_file.into())
                .map_err(|_| EraValidateError::InvalidMasterAccumulatorFile)?
        }
        None => MasterAccumulator::default(),
    };

    // Verify inclusion proof
    let blocks = extract_100_blocks(&directory, start_block, end_block)?;
    for block_idx in 0..num_blocks {
        let bhp = BlockHeaderProof::AccumulatorProof(AccumulatorProof {
            proof: inclusion_proof[block_idx].clone(),
        });
        let hwp = HeaderWithProof {
            header: header_from_block(blocks[block_idx].clone())?,
            proof: bhp,
        };
        master_accumulator
            .validate_header_with_proof(&hwp)
            .map_err(|_| EraValidateError::ProofGenerationFailure)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    // Test inclusion proof
    #[test]
    fn test_inclusion_proof() {
        let directory = String::from("./src/assets/ethereum_firehose_first_8200");
        let start_block = 301;
        let end_block = 402;
        let inclusion_proof = generate_inclusion_proof(&directory, start_block, end_block).unwrap();
        assert_eq!(inclusion_proof.len(), end_block - start_block);

        // Verify inclusion proof
        assert!(
            verify_inclusion_proof(&directory, None, start_block, end_block, inclusion_proof)
                .is_ok()
        );
    }
}
