// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Generates proof for block based on its relation to the Merge and Capella upgrades
//! in case of Ethereum BLocks. For Arbitrum, Optimism, it uses other methods to generate proofs

//use crate::protos::EthBlock;
use alloy_primitives::B256;
use ethportal_api::types::execution::header_with_proof::{
    BlockHeaderProof,
    // HistoricalRootsBlockProof, HistoricalSummariesBlockProof,
    PreMergeAccumulatorProof,
};

/// The merge block, inclusive, i.e., the block number below already counts as post-merge.
pub const MERGE_BLOCK: u64 = 15537394;
/// The first block after Shanghai-Capella block
pub const CAPELLA_START_BLOCK: u64 = 17_034_870;

/// A trait for EVM-based blockchains (Ethereum, Arbitrum, Optimism, etc.).
pub trait AnyBlock {
    /// return height of given block
    fn block_number(&self) -> u64;
    /// Returns the chain id
    fn chain_id(&self) -> EvmChain;
    /// Generates a proof for the block
    fn prove_block(&self) -> BlockHeaderProof;
}

/// Enum to differentiate which EVM chain it is.
#[derive(Debug)]
pub enum EvmChain {
    /// layer 1 ethereum
    Ethereum,
    ///arbiturm, layer 2
    Arbitrum,
    ///optimism layer 2
    Optimism,
}

/// Enum to differentiate Non-EVM chains.
/// Currently only Solana, but can be extended to include more.
#[derive(Debug)]
pub enum NonEvmChain {
    ///Solana type
    Solana,
    // Future chains can be added here (e.g., Aptos, Sui)
}

/// Represents a blockchain block that can be either an EVM block or a Non-EVM block.
///
/// This enum allows for storing different blockchain block types while maintaining a common interface.
/// It uses generics to store any type that implements the `AnyBlock` trait and provides
/// a separate variant for Non-EVM chains.
///
/// # Variants
/// - `Evm(E)`: Stores an EVM-based block (Ethereum, Arbitrum, Optimism, etc.).
/// - `NonEvm(NonEvmChain)`: Represents a block from a non-EVM chain.
pub enum Block<E: AnyBlock> {
    /// An EVM-based block, such as Ethereum, Arbitrum, or Optimism.
    Evm(E),
    /// A Non-EVM blockchain block (e.g., Solana, Sui, Aptos).
    NonEvm(NonEvmChain),
}

impl<E: AnyBlock> Block<E> {
    /// Retrieves the block number of the stored block.
    ///
    /// - Returns `Some(block_number)` for EVM-based blocks.
    /// - Returns `None` for Non-EVM blocks, as they may not have numeric block heights.
    ///
    /// # Example
    /// ```
    /// let eth_block = Block::Evm(EthereumBlock { number: 15537394 });
    /// assert_eq!(eth_block.block_number(), Some(15537394));
    ///
    /// let solana_block = Block::NonEvm(NonEvmChain::Solana);
    /// assert_eq!(solana_block.block_number(), None);
    /// ```
    pub fn block_number(&self) -> Option<u64> {
        match self {
            Block::Evm(block) => Some(block.block_number()),
            Block::NonEvm(_) => None, // Non-EVM chains don't necessarily use block numbers.
        }
    }

    /// Generates a proof for the stored block.
    ///
    /// - Returns `Some(BlockHeaderProof)` for EVM-based blocks.
    /// - Returns `None` for Non-EVM blocks, as proof mechanisms differ.
    ///
    /// # Example
    /// ```
    /// let eth_block = Block::Evm(EthereumBlock { number: 15537394 });
    /// let proof = eth_block.prove_block();
    /// assert!(proof.is_some());
    ///
    /// let solana_block = Block::NonEvm(NonEvmChain::Solana);
    /// let proof = solana_block.prove_block();
    /// assert!(proof.is_none());
    /// ```
    pub fn prove_block(&self) -> Option<BlockHeaderProof> {
        match self {
            Block::Evm(block) => Some(block.prove_block()),
            Block::NonEvm(_) => None, // Non-EVM proof logic would go here.
        }
    }

    /// Retrieves the chain type of the stored block.
    ///
    /// - Returns `Some(EvmChain)` for EVM blocks (Ethereum, Arbitrum, Optimism).
    /// - Returns `None` for Non-EVM chains.
    ///
    /// # Example
    /// ```
    /// let eth_block = Block::Evm(EthereumBlock { number: 15537394 });
    /// assert_eq!(eth_block.chain_id(), Some(EvmChain::Ethereum));
    ///
    /// let solana_block = Block::NonEvm(NonEvmChain::Solana);
    /// assert_eq!(solana_block.chain_id(), None);
    /// ```
    pub fn chain_id(&self) -> Option<EvmChain> {
        match self {
            Block::Evm(block) => Some(block.chain_id()),
            Block::NonEvm(_) => None,
        }
    }
}

/// Implement AnyBlock for EthereumBlock
struct EthereumBlock {
    pub number: u64,
}

impl AnyBlock for EthereumBlock {
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

/// Implement AnyBlock for ArbBlock
struct ArbBlock {
    pub number: u64,
}

impl AnyBlock for ArbBlock {
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

/// Implement AnyBlock for OptimismBlock
struct OptimismBlock {
    pub number: u64,
}

impl AnyBlock for OptimismBlock {
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
        let arb_block = mock_eth_block();
        let block = Block::Evm(arb_block);
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
