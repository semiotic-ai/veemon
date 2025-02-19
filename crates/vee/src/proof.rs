/// generatesp proof for block based on its relation to the Merge and Capella upgrades
/// in case of Ethereum BLocks. For Arbitrum, Optimism, it uses other methods to generate proofs
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

/// A trait that defines the common interface for different blockchain types
/// (e.g., Ethereum, Arbitrum, Optimism, Solana).
pub trait BlockEntity {
    fn block_number(&self) -> u64;
    fn prove_block(&self) -> BlockHeaderProof;
}

/// The generic Block struct that can support multiple block types.
pub struct Block<E: BlockEntity> {
    pub block: E, // The block of any blockchain type that implements BlockEntity
}

impl<E: BlockEntity> Block<E> {
    /// Prove the block based on its relation to the Merge and Capella upgrades.
    pub fn prove_block(&self) {
        let execution_block_number = self.block.block_number();

        if execution_block_number < MERGE_BLOCK {
            println!("Pre-Merge block: {:?}", execution_block_number);
            let _proof = self.block.prove_block();
        }
        //TODO: actually use correct capella block number
        else if execution_block_number < CAPELLA_START_SLOT as u64 {
            println!(
                "Post-Merge, Pre-Capella block: {:?}",
                execution_block_number
            );
        } else {
            println!("Post-Capella block: {:?}", execution_block_number);
        }
    }
}

/// Ethereum block implementation of BlockEntity trait
impl BlockEntity for EthBlock {
    fn block_number(&self) -> u64 {
        self.number
    }

    fn prove_block(&self) -> BlockHeaderProof {
        let proof = PreMergeAccumulatorProof {
            proof: [B256::default(); 15], // Example proof
        };

        BlockHeaderProof::PreMergeAccumulatorProof(proof)
    }
}

/// Arbitrum Block Implementation of BlockEntity trait
pub struct ArbBlock {
    pub number: u64,
}

impl BlockEntity for ArbBlock {
    fn block_number(&self) -> u64 {
        self.number
    }

    fn prove_block(&self) -> BlockHeaderProof {
        // Arbitrum-specific proof generation logic
        // Example, can differ from Ethereum's logic
        println!("Proving Arbitrum block: {:?}", self.number);
        BlockHeaderProof::PreMergeAccumulatorProof(PreMergeAccumulatorProof {
            proof: [B256::default(); 15], // Example proof for Arbitrum
        })
    }
}

/// Optimism Block Implementation of BlockEntity trait
pub struct OptimismBlock {
    pub number: u64,
    // other fields specific to Optimism block
}

impl BlockEntity for OptimismBlock {
    fn block_number(&self) -> u64 {
        self.number
    }

    fn prove_block(&self) -> BlockHeaderProof {
        // Optimism-specific proof generation logic
        println!("Proving Optimism block: {:?}", self.number);
        BlockHeaderProof::PreMergeAccumulatorProof(PreMergeAccumulatorProof {
            proof: [B256::default(); 15], // Example proof for Optimism
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_eth_block() -> EthBlock {
        EthBlock { number: 15537393 }
    }

    fn mock_arb_block() -> ArbBlock {
        ArbBlock { number: 15537395 } // After Merge, before Capella
    }

    fn mock_optimism_block() -> OptimismBlock {
        OptimismBlock { number: 15537400 }
    }

    #[test]
    fn test_prove_eth_block_pre_merge() {
        let eth_block = mock_eth_block();
        let block = Block { block: eth_block };

        block.prove_block();
    }

    #[test]
    fn test_prove_arb_block_post_merge_pre_capella() {
        let arb_block = mock_arb_block();
        let block = Block { block: arb_block };

        block.prove_block();
    }

    #[test]
    fn test_prove_optimism_block_post_capella() {
        let optimism_block = mock_optimism_block();
        let block = Block {
            block: optimism_block,
        };

        block.prove_block();
    }
}
