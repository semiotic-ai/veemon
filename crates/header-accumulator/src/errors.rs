// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use alloy_primitives::map::HashSet;
use firehose_protos::ProtosError;

/// Possible errors while interacting with the lib
#[derive(thiserror::Error, Debug)]
pub enum EraValidateError {
    /// Error decoding header from flat files
    #[error("Error decoding header from flat files")]
    HeaderDecodeError,

    /// Era accumulator mismatch
    #[error("Era accumulator mismatch")]
    EraAccumulatorMismatch,

    /// Block epoch mismatch
    #[error("Block epoch {block_epoch} (block number {block_number}) could not be proven with provided epoch {epoch_number}.")]
    EpochNotMatchForHeader {
        /// Epoch number
        epoch_number: usize,
        /// Block number
        block_number: u64,
        /// Block epoch
        block_epoch: usize,
    },

    /// Epoch not found in provided list
    #[error("Expected epoch {block_epoch} was not found in the provided epoch list. Epochs provided: {epoch_list:?}.")]
    EpochNotFoundInProvidedList {
        /// Block epoch
        block_epoch: usize,
        /// Provided epoch list
        epoch_list: Vec<usize>,
    },

    /// Error generating inclusion proof
    #[error("Error generating inclusion proof")]
    ProofGenerationFailure,

    /// Error validating inclusion proof
    #[error("Error validating inclusion proof")]
    ProofValidationFailure,

    /// Invalid epoch length
    #[error("Blocks in epoch must be exactly 8192 units, found {0}")]
    InvalidEpochLength(usize),

    /// Missing block in epoch
    #[error("Block was missing while creating epoch {epoch}. Missing blocks: {blocks:?}")]
    MissingBlock {
        /// Epoch number
        epoch: u64,
        /// Missing blocks
        blocks: Vec<u64>,
    },

    /// Invalid block in epoch
    #[error("Not all blocks are in the same epoch. Epochs found: {0:?}")]
    InvalidBlockInEpoch(HashSet<u64>),

    /// Error converting ExtHeaderRecord to header block number
    #[error("Error converting ExtHeaderRecord to header block number {0}")]
    ExtHeaderRecordError(u64),

    /// Invalid block range
    #[error("Invalid block range: {0} - {1}")]
    InvalidBlockRange(u64, u64),

    /// Epoch is in post merge
    #[error("epoch is in post merge: {0}")]
    EpochPostMerge(usize),

    /// header block number is different than expected
    #[error("header block number ({block_number}) is different than expected ({expected_number})")]
    HeaderMismatch {
        /// expected block number
        expected_number: u64,
        /// actual block number
        block_number: u64,
    },
    /// The proof does not match the expected era for the given header timestamp.
    #[error("proof type does not match expected era for timestamp {timestamp}")]
    InvalidProofEra {
        /// The timestamp of the block header being validated.
        timestamp: u64,
    },
}

impl From<ProtosError> for EraValidateError {
    fn from(error: ProtosError) -> Self {
        match error {
            ProtosError::BlockConversionError => Self::HeaderDecodeError,
            _ => unimplemented!("Error mapping is not implemented"),
        }
    }
}
