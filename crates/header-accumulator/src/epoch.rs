use std::array::IntoIter;

use crate::{errors::EraValidateError, types::ExtHeaderRecord};

/// The maximum number of slots per epoch in Ethereum.
/// In the context of Proof of Stake (PoS) consensus, an epoch is a collection of slots
/// during which validators propose and attest to blocks. The maximum size of an epoch
/// defines the number of slots that can be included in one epoch.
pub const MAX_EPOCH_SIZE: usize = 8192;

/// The final epoch number before the Ethereum network underwent "The Merge."
/// "The Merge" refers to the event where Ethereum transitioned from Proof of Work (PoW)
/// to Proof of Stake (PoS). The final epoch under PoW was epoch 1896.
pub const FINAL_EPOCH: usize = 1896;

/// The block number at which "The Merge" occurred in the Ethereum network.
/// "The Merge" took place at block 15537394, when the Ethereum network fully switched
/// from Proof of Work (PoW) to Proof of Stake (PoS).
pub const MERGE_BLOCK: u64 = 15537394;

/// Epoch containing 8192 blocks
pub struct Epoch(Box<[ExtHeaderRecord; MAX_EPOCH_SIZE]>);

impl TryFrom<Vec<ExtHeaderRecord>> for Epoch {
    type Error = EraValidateError;

    fn try_from(value: Vec<ExtHeaderRecord>) -> Result<Self, Self::Error> {
        let len = value.len();
        println!("length: {len}");
        let value: Vec<ExtHeaderRecord> = value.into_iter().take(MAX_EPOCH_SIZE).collect();
        let value: Box<[ExtHeaderRecord; MAX_EPOCH_SIZE]> = value
            .try_into()
            .map_err(|_| EraValidateError::InvalidEpochLength(len))?;
        let epoch_number = value[0].block_number / MAX_EPOCH_SIZE as u64;
        if value
            .iter()
            .map(|block| block.block_number / MAX_EPOCH_SIZE as u64)
            .all(|epoch| epoch == epoch_number)
        {
            Ok(Self(value))
        } else {
            Err(EraValidateError::InvalidBlockInEpoch)
        }
    }
}

impl Epoch {
    pub fn number(&self) -> usize {
        (self.0[0].block_number / MAX_EPOCH_SIZE as u64) as usize
    }
    pub fn iter(&self) -> std::slice::Iter<'_, ExtHeaderRecord> {
        self.0.iter()
    }
}

impl IntoIterator for Epoch {
    type Item = ExtHeaderRecord;
    type IntoIter = IntoIter<ExtHeaderRecord, MAX_EPOCH_SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
