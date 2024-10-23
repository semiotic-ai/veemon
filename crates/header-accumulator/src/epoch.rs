use std::array::IntoIter;

use alloy_primitives::map::HashSet;
use ethportal_api::types::execution::accumulator::HeaderRecord;

use crate::{errors::EraValidateError, types::ExtHeaderRecord};

/// The maximum number of slots per epoch in Ethereum.
///
/// In the context of Proof of Stake (PoS) consensus, an epoch is a collection of slots
/// during which validators propose and attest to blocks. The maximum size of an epoch
/// defines the number of slots that can be included in one epoch.
pub const MAX_EPOCH_SIZE: usize = 8192;

/// The final epoch number before the Ethereum network underwent "The Merge."
///
/// "The Merge" refers to the event where Ethereum transitioned from Proof of Work (PoW)
/// to Proof of Stake (PoS). The final epoch under PoW was epoch 1896.
pub const FINAL_EPOCH: usize = 1896;

/// The block number at which "The Merge" occurred in the Ethereum network.
///
/// "The Merge" took place at block 15537394, when the Ethereum network fully switched
/// from Proof of Work (PoW) to Proof of Stake (PoS).
pub const MERGE_BLOCK: u64 = 15537394;

/// Epoch containing 8192 blocks
///
/// An epoch must respect the order of blocks, i.e., block numbers for epoch
/// 0 must start from block 0 to block 8191.
///
/// All blocks must be at the same epoch
pub struct Epoch {
    number: usize,
    data: Box<[HeaderRecord; MAX_EPOCH_SIZE]>,
}

impl TryFrom<Vec<ExtHeaderRecord>> for Epoch {
    type Error = EraValidateError;

    fn try_from(mut data: Vec<ExtHeaderRecord>) -> Result<Self, Self::Error> {
        // all data must be sorted
        data.sort_by(|b1, b2| b1.block_number.cmp(&b2.block_number));
        // max MAX_EPOCH_SIZE in the array
        data.truncate(MAX_EPOCH_SIZE);
        let len = data.len();
        // get the first block to get the block number
        let epoch_number = data
            .first()
            .map(|block| block.block_number / MAX_EPOCH_SIZE as u64)
            .ok_or(EraValidateError::InvalidEpochLength(0))?;
        // cannot have any missing blocks
        let blocks_missing: Vec<_> = data
            .windows(2)
            .filter(|w| w[1].block_number - w[0].block_number != 1)
            .map(|w| w[0].block_number + 1)
            .collect();
        if !blocks_missing.is_empty() {
            return Err(EraValidateError::MissingBlock {
                blocks: blocks_missing,
                epoch: epoch_number,
            });
        }

        // check if all blocks are in the same era
        let epochs_found: HashSet<u64> = data
            .iter()
            .map(|block| block.block_number / MAX_EPOCH_SIZE as u64)
            .collect();
        if epochs_found.len() > 1 {
            return Err(EraValidateError::InvalidBlockInEpoch(epochs_found));
        }
        let data: Box<[HeaderRecord]> = data.into_iter().map(Into::into).collect();
        let data: Box<[HeaderRecord; MAX_EPOCH_SIZE]> = data
            .try_into()
            .map_err(|_| EraValidateError::InvalidEpochLength(len))?;
        Ok(Self {
            number: epoch_number as usize,
            data,
        })
    }
}

impl Epoch {
    pub fn number(&self) -> usize {
        self.number
    }

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
