// SPDX-FileCopyrightText: 2024- Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::array::IntoIter;

use alloy_consensus::Header;
use alloy_primitives::{Uint, B256};
use ethportal_api::types::execution::accumulator::{EpochAccumulator, HeaderRecord};
use firehose_protos::{BlockHeader, EthBlock as Block, ProtosError};

use crate::error::EraValidationError;
use crate::types::{BlockNumber, EpochNumber};

/// the maximum number of slots per epoch in ethereum.
///
/// in the context of proof of stake (pos) consensus, an epoch is a collection of slots
/// during which validators propose and attest to blocks. the maximum size of an epoch
/// defines the number of slots that can be included in one epoch.
pub const MAX_EPOCH_SIZE: usize = 8192;

/// the final epoch number before the ethereum network underwent "the merge."
///
/// "the merge" refers to the event where ethereum transitioned from proof of work (pow)
/// to proof of stake (pos). the final epoch under pow was epoch 1896.
pub const FINAL_EPOCH: usize = 1896;

/// the block number at which "the merge" occurred in the ethereum network.
///
/// "the merge" took place at block 15537394, when the ethereum network fully switched
/// from proof of work (pow) to proof of stake (pos).
pub const MERGE_BLOCK: u64 = 15537394;

/// epoch containing 8192 blocks
///
/// an epoch must respect the order of blocks, i.e., block numbers for epoch
/// 0 must start from block 0 to block 8191.
///
/// all blocks must be at the same epoch
#[derive(Clone)]
pub struct Epoch {
    number: EpochNumber,
    data: Box<[HeaderRecord; MAX_EPOCH_SIZE]>,
}

impl TryFrom<Vec<ExtHeaderRecord>> for Epoch {
    type Error = EraValidationError;

    fn try_from(mut data: Vec<ExtHeaderRecord>) -> Result<Self, Self::Error> {
        // all data must be sorted
        data.sort_by(|b1, b2| b1.block_number.cmp(&b2.block_number));
        // max MAX_EPOCH_SIZE in the array
        data.truncate(MAX_EPOCH_SIZE);
        let len = data.len();
        // get the first block to get the block number
        let epoch_number: EpochNumber = data
            .first()
            .map(|block| block.block_number.into())
            .ok_or(EraValidationError::InvalidEpochLength(0))?;
        // cannot have any missing blocks
        let blocks_missing: Vec<BlockNumber> = data
            .windows(2)
            .filter(|w| (w[1].block_number.0 - w[0].block_number.0) != 1)
            .map(|w| BlockNumber(w[0].block_number.0 + 1))
            .collect();
        if !blocks_missing.is_empty() {
            return Err(EraValidationError::MissingBlock {
                blocks: blocks_missing,
                epoch: epoch_number,
            });
        }

        // check if all blocks are in the same era
        let mut epochs_found: Vec<EpochNumber> =
            data.iter().map(|block| block.block_number.into()).collect();
        epochs_found.sort_unstable();
        epochs_found.dedup();
        if epochs_found.len() > 1 {
            return Err(EraValidationError::InvalidBlockInEpoch(epochs_found));
        }
        let data: Box<[HeaderRecord]> = data.into_iter().map(Into::into).collect();
        let data: Box<[HeaderRecord; MAX_EPOCH_SIZE]> = data
            .try_into()
            .map_err(|_| EraValidationError::InvalidEpochLength(len as u64))?;
        Ok(Self {
            number: epoch_number,
            data,
        })
    }
}

impl From<Epoch> for EpochAccumulator {
    fn from(value: Epoch) -> Self {
        let vec: Vec<HeaderRecord> = value.data.to_vec();
        EpochAccumulator::from(vec)
    }
}

impl Epoch {
    /// get the epoch number
    pub fn number(&self) -> EpochNumber {
        self.number
    }

    /// get an iterator over the epoch data
    pub fn iter(&self) -> std::slice::Iter<'_, HeaderRecord> {
        self.data.iter()
    }
}

impl IntoIterator for Epoch {
    type Item = HeaderRecord;
    type IntoIter = IntoIter<Self::Item, MAX_EPOCH_SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

/// extension of header with conversion traits
///
/// it's capable of converting to HeaderRecord to be used inside Epochs.
/// it can also convert to firehose protos.
/// you can extract the full header as an option
#[derive(Clone)]
pub struct ExtHeaderRecord {
    /// block hash
    pub block_hash: B256,
    /// total difficulty
    pub total_difficulty: Uint<256, 4>,
    /// block number
    pub block_number: BlockNumber,
    /// full header
    pub full_header: Option<Header>,
}

impl From<ExtHeaderRecord> for HeaderRecord {
    fn from(
        ExtHeaderRecord {
            block_hash,
            total_difficulty,
            ..
        }: ExtHeaderRecord,
    ) -> Self {
        HeaderRecord {
            block_hash,
            total_difficulty,
        }
    }
}

impl TryFrom<ExtHeaderRecord> for Header {
    type Error = EraValidationError;

    fn try_from(ext: ExtHeaderRecord) -> Result<Self, Self::Error> {
        ext.full_header
            .ok_or(EraValidationError::ExtHeaderRecordError(ext.block_number))
    }
}

impl From<&ExtHeaderRecord> for HeaderRecord {
    fn from(ext: &ExtHeaderRecord) -> Self {
        HeaderRecord {
            block_hash: ext.block_hash,
            total_difficulty: ext.total_difficulty,
        }
    }
}

/// decodes a [`ExtHeaderRecord`] from a [`Block`]. a [`BlockHeader`] must be present in the block,
/// otherwise validating headers won't be possible
impl TryFrom<&Block> for ExtHeaderRecord {
    type Error = EraValidationError;

    fn try_from(block: &Block) -> Result<Self, Self::Error> {
        let header: &BlockHeader =
            block
                .header
                .as_ref()
                .ok_or(EraValidationError::HeaderDecode(
                    ProtosError::BlockHeaderMissing,
                ))?;

        let total_difficulty =
            header
                .total_difficulty
                .as_ref()
                .ok_or(EraValidationError::HeaderDecode(
                    ProtosError::BlockConversionError,
                ))?;

        Ok(ExtHeaderRecord {
            block_number: BlockNumber(block.number),
            block_hash: B256::from_slice(&block.hash),
            total_difficulty: Uint::from_be_slice(total_difficulty.bytes.as_slice()),
            full_header: Some(block.try_into()?),
        })
    }
}
