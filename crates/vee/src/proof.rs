use crate::protos::EthBlock;
use alloy_primitives::B256;
use ethportal_api::types::execution::header_with_proof::{
    BlockHeaderProof,
    // HistoricalRootsBlockProof, HistoricalSummariesBlockProof,
    PreMergeAccumulatorProof,
};

/// The maximum number of block roots that can be stored in a [`BeaconState`]'s `block_roots` list.
pub const SLOTS_PER_HISTORICAL_ROOT: usize = 8192;
/// The merge block, inclusive, i.e., the block number below already counts to be post-merge.
pub const MERGE_BLOCK: u64 = 15537394;
/// The number of slots in an epoch.
pub const SLOTS_PER_EPOCH: usize = 32;
/// The number of slots in an era.
pub const SLOTS_PER_ERA: usize = SLOTS_PER_HISTORICAL_ROOT;
/// Slots are 0-indexed.
/// See, for example, `https://beaconcha.in/slot/0`.
pub const CAPELLA_START_EPOCH: usize = 194048;
/// See [Upgrading Ethereum](https://eth2book.info/capella/part4/history/) for more information.
/// The first slot number of the Deneb fork.
pub const CAPELLA_START_SLOT: usize = CAPELLA_START_EPOCH * SLOTS_PER_EPOCH;
/// The first era of the Deneb fork.
pub const CAPELLA_START_ERA: usize =
    (CAPELLA_START_EPOCH * SLOTS_PER_EPOCH) / SLOTS_PER_HISTORICAL_ROOT;

/// generatesp proof for block based on its relation to the Merge and Capella upgrades.
/// This function receives an execution block and verifies accordingly.
pub fn prove_block(execution_block: &EthBlock) {
    let execution_block_number = execution_block.number;

    // Check if block is pre-merge
    if execution_block_number < MERGE_BLOCK {
        // Pre-Merge: Use the pre-Merge accumulator
        println!("Pre-Merge block: {:?}", execution_block_number);
        prove_pre_merge_block(execution_block);
    } else if execution_block_number < CAPELLA_START_SLOT as u64 {
        // Post-Merge, Pre-Capella: Use HistoricalBatch
        println!(
            "Post-Merge, Pre-Capella block: {:?}",
            execution_block_number
        );
        prove_pre_capella(execution_block);
    } else {
        // Post-Capella: Use HistoricalSummary
        println!("Post-Capella block: {:?}", execution_block_number);
        prove_post_capella(execution_block);
    }
}

/// Verifies a pre-Merge block using the pre-Merge accumulator.
fn prove_pre_merge_block(execution_block: &EthBlock) -> Result<BlockHeaderProof, String> {
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
fn prove_pre_capella(execution_block: &EthBlock) {
    // TODO: Implement post-Merge pre-Capella verification logic

    // TODO: build these proofs
    // let proof = HistoricalRootsBlockProof {
    //     proof: [B256::default(); 15], // Empty proof with default 256 values
    // };

    unimplemented!("Implement HistoricalBatch verification");
}

/// Verifies a post-Capella block using the HistoricalSummary.
fn prove_post_capella(execution_block: &EthBlock) {
    // TODO: Implement post-Capella verification logic

    //TODO: build these proofs
    // let proof = HistoricalSummariesBlockProof {
    // };
    unimplemented!("Implement HistoricalSummary verification");
}
