// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Generates proof for block based on its relation to the Merge and Capella upgrades
//! in case of Ethereum BLocks. For Arbitrum, Optimism, it uses other methods to generate proofs

use crate::protos::EthBlock;
use alloy_primitives::B256;
use ethportal_api::types::execution::header_with_proof::{
    BlockHeaderProof,
    // HistoricalRootsBlockProof, HistoricalSummariesBlockProof,
    PreMergeAccumulatorProof,
};
// use header_accumulator::{
//     self, // generate_inclusion_proof
// };

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
    pub fn chain_id(&self) -> Option<EvmChain> {
        match self {
            Block::Evm(block) => Some(block.chain_id()),
            Block::NonEvm(_) => None,
        }
    }
}

/// Implement AnyBlock for EthereumBlock
pub struct EthereumBlock(pub EthBlock);

impl AnyBlock for EthereumBlock {
    fn block_number(&self) -> u64 {
        self.0.number
    }

    fn chain_id(&self) -> EvmChain {
        EvmChain::Ethereum
    }

    /// Generates a Merkle proof for the current block header depending on which phase
    /// of Ethereum's chain history the block belongs to: pre-Merge, post-Merge (pre-Capella),
    /// or post-Capella.
    ///
    /// Ethereum underwent two key transitions:
    /// - The **Merge** at block `15_537_394`, switching from PoW to PoS.
    /// - The **Capella** fork at block `17_034_870`, enabling validator withdrawals.
    ///
    /// The Portal Network's **historical header accumulator** divides chain history into
    /// fixed-size "eras" of 8192 slot-groups each. These eras start **at the Merge block**
    /// (era 573) and extend through to **era 757**, which ends at block `17_052_913`.
    ///
    /// This means:
    /// - from pre-merge, epoch 0 to epoch 1896 marks pre-merge blocks.
    /// - **Era 573 starts at block 15_537_394** (the Merge block).
    /// - **Era 757 ends at block 17_052_913**, which is **after Capella on the execution layer by 18043 blocks**
    ///   (for reference, the Capella block fork start is in  : 17_034_870)
    ///
    ///  Therefore, the Portal pre-Capella accumulator contains **some post-Capella EXECUTION blocks**.
    ///
    fn prove_block(&self) -> BlockHeaderProof {
        let execution_block_number = self.block_number();

        if execution_block_number < MERGE_BLOCK {
            todo!()
        //TODO: the epoch of 8192 blocks is necessary here, to generate a proof. Get it with
        // the firehoseClilent for now. But given it is too many blocks, maybe later store ina a buffer
        // for reuse
        } else if execution_block_number < CAPELLA_START_BLOCK
            && execution_block_number > MERGE_BLOCK
        {
            println!(
                "Post-Merge, Pre-Capella Ethereum block: {:?}",
                execution_block_number
            );
            todo!()
        }

        println!("Post-Capella Ethereum block: {:?}", execution_block_number);
        todo!()
    }
}

/// Implement AnyBlock for ArbBlock
#[allow(dead_code)]
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
#[allow(dead_code)]
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

    //TODO: import a block from assets for proving it
    // fn mock_ethereum_block(number: u64) -> EthereumBlock {
    //     EthereumBlock(EthBlock { number }) // Ensure EthBlock struct has the required field
    // }

    fn mock_arb_block() -> ArbBlock {
        ArbBlock { number: 15537395 }
    }

    fn mock_optimism_block() -> OptimismBlock {
        OptimismBlock { number: 15537400 }
    }

    // #[test]
    // fn test_prove_eth_block_pre_merge() {
    //     let eth_block = mock_ethereum_block(15537393); // Pre-merge block
    //     let block = Block::Evm(eth_block);
    //
    //     let proof = block.prove_block();
    //     assert!(proof.is_some()); // Ensure proof generation doesn't fail
    // }
    //
    //
    #[test]
    fn test_prove_arb_block_post_merge_pre_capella() {
        let arb_block = mock_arb_block();
        let block = Block::Evm(arb_block);

        let proof = block.prove_block();
        assert!(proof.is_some()); // Ensure proof is generated
    }

    #[test]
    fn test_prove_optimism_block_post_capella() {
        let optimism_block = mock_optimism_block();
        let block = Block::Evm(optimism_block);

        let proof = block.prove_block();
        assert!(proof.is_some()); // Ensure proof is generated
    }
}
