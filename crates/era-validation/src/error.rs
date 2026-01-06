// SPDX-FileCopyrightText: 2024- Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use firehose_protos::ProtosError;
use primitive_types::H256;

use crate::types::{BlockNumber, EpochNumber, EraNumber, SlotNumber};

/// Unified era validation error type for all blockchain eras and chains
#[derive(thiserror::Error, Debug)]
pub enum EraValidationError {
    // Ethereum Pre-Merge errors
    #[error("ethereum pre-merge validation failed: {0}")]
    EthereumPreMerge(#[from] EthereumPreMergeError),

    // Ethereum Post-Merge errors
    #[error("ethereum post-merge validation failed: {0}")]
    EthereumPostMerge(#[from] EthereumPostMergeError),

    // Ethereum Post-Capella errors
    #[error("ethereum post-capella validation failed: {0}")]
    EthereumPostCapella(#[from] EthereumPostCapellaError),

    // Solana errors
    #[error("solana validation failed: {0}")]
    Solana(#[from] SolanaValidatorError),

    // Epoch/Era errors
    #[error("epoch is in post merge: {0}")]
    EpochPostMerge(EpochNumber),

    #[error("blocks in epoch must be exactly 8192 units, found {0}")]
    InvalidEpochLength(u64),

    #[error("block was missing while creating epoch {epoch}. missing blocks: {blocks:?}")]
    MissingBlock {
        /// Epoch number
        epoch: EpochNumber,
        /// Missing blocks
        blocks: Vec<BlockNumber>,
    },

    #[error("not all blocks are in the same epoch. epochs found: {0:?}")]
    InvalidBlockInEpoch(Vec<EpochNumber>),

    #[error("block epoch {block_epoch} (block number {block_number}) could not be proven with provided epoch {epoch_number}.")]
    EpochNotMatchForHeader {
        /// Epoch number
        epoch_number: EpochNumber,
        /// Block number
        block_number: BlockNumber,
        /// Block epoch
        block_epoch: EpochNumber,
    },

    #[error("expected epoch {block_epoch} was not found in the provided epoch list. epochs provided: {epoch_list:?}.")]
    EpochNotFoundInProvidedList {
        /// Block epoch
        block_epoch: EpochNumber,
        /// Provided epoch list
        epoch_list: Vec<EpochNumber>,
    },

    // Proof errors
    #[error("error generating inclusion proof")]
    ProofGenerationFailure,

    #[error("error validating inclusion proof")]
    ProofValidationFailure,

    // Header/Block errors
    #[error("error decoding header from flat files: {0}")]
    HeaderDecode(#[source] ProtosError),

    #[error("error converting ExtHeaderRecord to header block number {0}")]
    ExtHeaderRecordError(BlockNumber),

    #[error("header block number ({block_number}) is different than expected ({expected_number})")]
    HeaderMismatch {
        /// Expected block number
        expected_number: BlockNumber,
        /// Actual block number
        block_number: BlockNumber,
    },

    #[error("invalid block range: {0} - {1}")]
    InvalidBlockRange(BlockNumber, BlockNumber),

    // Accumulator errors
    #[error("era accumulator mismatch")]
    EraAccumulatorMismatch,
}

/// Ethereum pre-merge specific errors
#[derive(thiserror::Error, Debug)]
pub enum EthereumPreMergeError {
    #[error("number of execution block hashes must match the epoch length")]
    MismatchedBlockCount,

    #[error("invalid historical root for era {era}: expected {expected}, got {actual}")]
    InvalidHistoricalRoot {
        era: EpochNumber,
        expected: H256,
        actual: H256,
    },

    #[error("epoch {epoch} is out of bounds (maximum valid epoch: {max_epoch})")]
    EpochOutOfBounds {
        epoch: EpochNumber,
        max_epoch: EpochNumber,
    },
}

/// Common errors for Ethereum PoS eras (post-merge and post-Capella)
#[derive(thiserror::Error, Debug, Clone)]
pub enum EthereumPosEraError {
    #[error("number of execution block hashes must match the number of beacon blocks")]
    MismatchedBlockCount,

    #[error("execution block hash mismatch: expected {expected:?}, got {actual:?}")]
    ExecutionBlockHashMismatch {
        expected: Option<H256>,
        actual: Option<H256>,
    },

    #[error("invalid era start: slot {0} is not a multiple of 8192")]
    InvalidEraStart(SlotNumber),

    #[error("invalid block summary root for era {era}: expected {expected}, got {actual}")]
    InvalidBlockSummaryRoot {
        era: EraNumber,
        expected: H256,
        actual: H256,
    },

    #[error("era {era} is out of bounds (maximum valid era: {max_era})")]
    EraOutOfBounds { era: EraNumber, max_era: EraNumber },
}

/// Ethereum post-merge (pre-Capella) specific errors
#[derive(thiserror::Error, Debug)]
pub enum EthereumPostMergeError {
    #[error(transparent)]
    Common(#[from] EthereumPosEraError),
}

/// Ethereum post-Capella specific errors
#[derive(thiserror::Error, Debug)]
pub enum EthereumPostCapellaError {
    #[error(transparent)]
    Common(#[from] EthereumPosEraError),
}

/// Solana specific errors
#[derive(thiserror::Error, Debug)]
pub enum SolanaValidatorError {
    #[error("number of execution block hashes must match the epoch length")]
    MismatchedBlockCount,

    #[error("invalid historical root for era {era}: expected {expected}, got {actual}")]
    InvalidHistoricalRoot {
        era: EpochNumber,
        expected: H256,
        actual: H256,
    },

    #[error("epoch {epoch} is out of bounds (maximum valid epoch: {max_epoch})")]
    EpochOutOfBounds {
        epoch: EpochNumber,
        max_epoch: EpochNumber,
    },
}

impl From<ProtosError> for EraValidationError {
    fn from(error: ProtosError) -> Self {
        EraValidationError::HeaderDecode(error)
    }
}
