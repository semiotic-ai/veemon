use alloy_primitives::B256;
use firehose_protos::ethereum_v2::{Block, BlockHeader};
use serde::{Deserialize, Serialize};

use crate::error::DecoderError;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeaderRoots {
    pub receipt_root: B256,
    pub transactions_root: B256,
}

impl TryFrom<&BlockHeader> for BlockHeaderRoots {
    type Error = DecoderError;

    fn try_from(header: &BlockHeader) -> Result<Self, Self::Error> {
        let receipt_root: [u8; 32] = header.receipt_root.as_slice().try_into()?;
        let transactions_root: [u8; 32] = header.transactions_root.as_slice().try_into()?;

        Ok(Self {
            receipt_root: receipt_root.into(),
            transactions_root: transactions_root.into(),
        })
    }
}

impl TryFrom<&Block> for BlockHeaderRoots {
    type Error = DecoderError;

    fn try_from(block: &Block) -> Result<Self, Self::Error> {
        block.header()?.try_into()
    }
}

impl BlockHeaderRoots {
    /// Check if the block header roots match those of self.
    /// All `Ok` results are considered a match.
    pub(crate) fn block_header_matches(&self, block: &Block) -> Result<bool, DecoderError> {
        let block_header_roots = block.try_into()?;

        if self.block_header_roots_match(&block_header_roots) {
            Ok(true)
        } else {
            Err(DecoderError::MismatchedRoots {
                block_number: block.number,
            })
        }
    }

    fn block_header_roots_match(&self, block_header_roots: &BlockHeaderRoots) -> bool {
        self == block_header_roots
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct HeaderRecordWithNumber {
    pub block_hash: Vec<u8>,
    pub total_difficulty: Vec<u8>,
    pub block_number: u64,
}

impl TryFrom<Block> for HeaderRecordWithNumber {
    type Error = DecoderError;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        let block_header = block.header()?;

        let header_record_with_number = HeaderRecordWithNumber {
            block_hash: block.hash.clone(),
            total_difficulty: block_header
                .total_difficulty
                .as_ref()
                .ok_or(Self::Error::InvalidTotalDifficulty)?
                .bytes
                .clone(),
            block_number: block.number,
        };
        Ok(header_record_with_number)
    }
}
