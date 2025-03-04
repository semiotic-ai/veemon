/*
generatesp proof for block based on its relation to the Merge and Capella upgrades
in case of Ethereum BLocks. For Arbitrum, Optimism, it uses other methods to generate proofs
*/
use crate::protos::EthBlock;
use alloy_primitives::B256;
use ethportal_api::types::execution::header_with_proof::{
    BlockHeaderProof,
    // HistoricalRootsBlockProof, HistoricalSummariesBlockProof,
    PreMergeAccumulatorProof,
};

/// The merge block, inclusive, i.e., the block number below already counts to be post-merge.
pub const MERGE_BLOCK: u64 = 15537394;
/// The first block after Shanghai-capella block
pub const CAPELLA_START_BLOCK: u64 = 17_034_870;

/// A trait for EVM-based blockchains (Ethereum, Arbitrum, Optimism, etc.)
pub trait EvmBlock {
    fn block_number(&self) -> u64;
    fn chain_id(&self) -> EvmChain;
    fn prove_block(&self) -> BlockHeaderProof;
}

/// Enum to differentiate which EVM chain it is
#[derive(Debug)]
pub enum EvmChain {
    Ethereum,
    Arbitrum,
    Optimism,
}

/// A trait for Solana-based blockchains
pub trait SolanaBlock {}

/// Define a `Block` enum that can store either an **EVM block (generic) or a Solana block**
pub enum Block<E: EvmBlock> {
    Evm(E),
    Solana(SolanaBlockImpl),
}

impl<E: EvmBlock> Block<E> {
    pub fn block_number(&self) -> Option<u64> {
        match self {
            Block::Evm(block) => Some(block.block_number()),
            Block::Solana(_) => None, // Solana blocks don't have block numbers
        }
    }

    pub fn prove_block(&self) -> Option<BlockHeaderProof> {
        match self {
            Block::Evm(block) => Some(block.prove_block()),
            Block::Solana(_) => None, // Solana proof logic would go here
        }
    }

    pub fn chain_id(&self) -> Option<EvmChain> {
        match self {
            Block::Evm(block) => Some(block.chain_id()),
            Block::Solana(_) => None,
        }
    }
}

/// Implement EvmBlock for EthereumBlock
pub struct EthereumBlock {
    pub number: u64,
}

impl EvmBlock for EthereumBlock {
    fn block_number(&self) -> u64 {
        self.number
    }

    fn chain_id(&self) -> EvmChain {
        EvmChain::Ethereum
    }

    fn prove_block(&self) -> BlockHeaderProof {
        let execution_block_number = self.block_number();

        if execution_block_number < MERGE_BLOCK {
            println!("Pre-Merge Ethereum block: {:?}", execution_block_number);
            todo!()
        } else if execution_block_number < CAPELLA_START_BLOCK {
            println!(
                "Post-Merge, Pre-Capella Ethereum block: {:?}",
                execution_block_number
            );
            todo!()
        } else {
            println!("Post-Capella Ethereum block: {:?}", execution_block_number);
            todo!()
        }
    }
}

/// Implement EvmBlock for ArbBlock
pub struct ArbBlock {
    pub number: u64,
}

impl EvmBlock for ArbBlock {
    fn block_number(&self) -> u64 {
        self.number
    }

    fn chain_id(&self) -> EvmChain {
        EvmChain::Arbitrum
    }

    fn prove_block(&self) -> BlockHeaderProof {
        println!("Proving Arbitrum block: {:?}", self.number);
        BlockHeaderProof::PreMergeAccumulatorProof(PreMergeAccumulatorProof {
            proof: [B256::default(); 15],
        })
    }
}

/// Implement EvmBlock for OptimismBlock
pub struct OptimismBlock {
    pub number: u64,
}

impl EvmBlock for OptimismBlock {
    fn block_number(&self) -> u64 {
        self.number
    }

    fn chain_id(&self) -> EvmChain {
        EvmChain::Optimism
    }

    fn prove_block(&self) -> BlockHeaderProof {
        println!("Proving Optimism block: {:?}", self.number);
        BlockHeaderProof::PreMergeAccumulatorProof(PreMergeAccumulatorProof {
            proof: [B256::default(); 15],
        })
    }
}

/// Implement SolanaBlock for SolanaBlockImpl
pub struct SolanaBlockImpl;

impl SolanaBlock for SolanaBlockImpl {}
// TODO: implement receipt trait
// where we can retrieve specific block data
// for generating the proof
// tip: check parquet nozzle files, for receipt related matadata that
// is filled in each row
// tip 2: metadata_db can have additional receipt information if configure to.

#[cfg(test)]
mod tests {

    use super::*;

    fn mock_eth_block() -> EthereumBlock {
        EthereumBlock {
            number: MERGE_BLOCK,
        }
    }

    fn mock_arb_block() -> ArbBlock {
        ArbBlock { number: 15537395 }
    }

    fn mock_optimism_block() -> OptimismBlock {
        OptimismBlock { number: 15537400 }
    }

    #[test]
    fn test_prove_eth_block_pre_merge() {
        let block = Block::Evm(EthereumBlock {
            number: MERGE_BLOCK,
        });
        block.prove_block();
    }

    #[test]
    fn test_prove_arb_block_post_merge_pre_capella() {
        let arb_block = mock_arb_block();
        let block = Block::Evm(arb_block);
        block.prove_block();
    }

    #[test]
    fn test_prove_optimism_block_post_capella() {
        let optimism_block = mock_optimism_block();
        let block = Block::Evm(optimism_block);
        block.prove_block();
    }
}
