use alloy_primitives::B256;
use ethportal_api::types::execution::header_with_proof::{
    BlockHeaderProof, HistoricalRootsBlockProof, HistoricalSummariesBlockProof,
    PreMergeAccumulatorProof,
};
use firehose_protos::EthBlock;
use reth_primitives::Block;
use types::BeaconBlock;

use crate::beacon_state::{CAPELLA_START_ERA, CAPELLA_START_SLOT, MERGE_BLOCK};

enum BlockVariant<E: types::EthSpec> {
    Beacon(BeaconBlock<E>),
    Execution(EthBlock),
    Both {
        beacon: BeaconBlock<E>,
        execution: EthBlock,
    },
}

pub struct Blocks<E: types::EthSpec> {
    block: BlockVariant<E>,
}

/// Verifies the block based on its relation to the Merge and Capella upgrades.
pub fn verify_block<E: types::EthSpec>(blocks: Blocks<E>) {
    match &blocks.block {
        BlockVariant::Execution(execution_block) => {
            let execution_block_number = execution_block.number;
            if execution_block_number < MERGE_BLOCK {
                // Pre-Merge: Use the pre-Merge accumulator
                println!("Pre-Merge block: {:?}", execution_block_number);
                verify_pre_merge_block(execution_block);
            }
        }
        BlockVariant::Beacon(beacon_block) => {
            if beacon_block.slot().as_u64() < CAPELLA_START_SLOT.try_into().unwrap() {
                // Post-Merge, Pre-Capella: Use HistoricalBatch
                println!("Post-Merge, Pre-Capella block: {:?}", beacon_block.slot());
                verify_post_merge_pre_capella_block(&blocks);
            } else {
                // Post-Capella: Use HistoricalSummary
                println!("Post-Capella block: {:?}", beacon_block.slot());
                verify_post_capella_block(&blocks);
            }

            println!(
                "Beacon block verification is currently unimplemented: {:?}",
                beacon_block
            );
        }
        BlockVariant::Both { beacon, execution } => {
            //TODO: when both present, check if the execution_payload matches the beacon block
            // There is a way to generate a proof for it
            println!(
                "Both Beacon and Execution blocks are provided: Beacon {:?}, Execution {:?}",
                beacon, execution
            );

            let execution_block_number = execution.number;

            if execution_block_number < MERGE_BLOCK {
                println!("Pre-Merge block: {:?}", execution_block_number);
                verify_pre_merge_block(execution);
            } else if execution_block_number >= MERGE_BLOCK
                && execution_block_number < CAPELLA_START_ERA.try_into().unwrap()
            {
                println!(
                    "Post-Merge, Pre-Capella block: {:?}",
                    execution_block_number
                );
                verify_post_merge_pre_capella_block(&blocks);
            } else {
                println!("Post-Capella block: {:?}", execution_block_number);
                verify_post_capella_block(&blocks);
            }
        }
    }
}

/// Verifies a pre-Merge block using the pre-Merge accumulator.
fn verify_pre_merge_block(execution_block: &EthBlock) -> Result<BlockHeaderProof, String> {
    // Ensure the block has the required number and hash fields

    // TODO: Replace this with actual logic to use the pre-Merge accumulator.
    // Emit an empty proof for now
    let proof = PreMergeAccumulatorProof {
        proof: [B256::default(); 15], // Empty proof with default B256 values
    };

    // Wrap the proof in BlockHeaderProof::PreMergeAccumulatorProof
    Ok(BlockHeaderProof::PreMergeAccumulatorProof(proof))
}
/// Verifies a post-Merge pre-Capella block using the HistoricalBatch.
fn verify_post_merge_pre_capella_block<E: types::EthSpec>(blocks: &Blocks<E>) {
    // TODO: Implement post-Merge pre-Capella verification logic

    // TODO: build these proofs
    // let proof = HistoricalRootsBlockProof {
    //     proof: [B256::default(); 15], // Empty proof with default 256 values
    // };

    unimplemented!("Implement HistoricalBatch verification");
}

/// Verifies a post-Capella block using the HistoricalSummary.
fn verify_post_capella_block<E: types::EthSpec>(blocks: &Blocks<E>) {
    // TODO: Implement post-Capella verification logic

    //TODO: build these proofs
    // let proof = HistoricalSummariesBlockProof {
    // };
    unimplemented!("Implement HistoricalSummary verification");
}
