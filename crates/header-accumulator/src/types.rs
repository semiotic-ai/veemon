use alloy_primitives::Uint;
use alloy_primitives::B256;
use ethportal_api::{types::execution::accumulator::HeaderRecord, Header};
use sf_protos::ethereum::r#type::v2::Block;

use crate::errors::EraValidateError;
use crate::utils::header_from_block;

#[derive(Clone)]
pub struct ExtHeaderRecord {
    pub block_hash: B256,
    pub total_difficulty: Uint<256, 4>,
    pub block_number: u64,
    pub full_header: Option<Header>,
}

impl From<ExtHeaderRecord> for HeaderRecord {
    fn from(ext: ExtHeaderRecord) -> Self {
        HeaderRecord {
            block_hash: ext.block_hash,
            total_difficulty: ext.total_difficulty,
        }
    }
}

impl From<ExtHeaderRecord> for Header {
    fn from(ext: ExtHeaderRecord) -> Self {
        ext.full_header.unwrap()
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

/// Decodes a [`ExtHeaderRecord`] from a [`Block`]. A [`BlockHeader`] must be present in the block,
/// otherwise validating headers won't be possible
impl TryFrom<&Block> for ExtHeaderRecord {
    type Error = EraValidateError; // Ensure this matches your error type

    fn try_from(block: &Block) -> Result<Self, Self::Error> {
        let header = block
            .header
            .as_ref()
            .ok_or(EraValidateError::HeaderDecodeError)?;
        let total_difficulty = header
            .total_difficulty
            .as_ref()
            .ok_or(EraValidateError::HeaderDecodeError)?;

        Ok(ExtHeaderRecord {
            block_number: block.number,
            block_hash: B256::from_slice(&block.hash),
            total_difficulty: Uint::from_be_slice(total_difficulty.bytes.as_slice()),
            full_header: Some(header_from_block(block)?), // Assuming header_from_block returns Result<_, EraValidateError>
        })
    }
}
