pub mod error;

use crate::headers::error::BlockHeaderError;
use firehose_protos::ethereum_v2::{eth_block::BlockHeaderRoots, Block};
use serde::{Deserialize, Serialize};

pub(crate) fn check_valid_header(
    block: &Block,
    header_roots: BlockHeaderRoots,
) -> Result<(), BlockHeaderError> {
    let block_header = match block.header.as_ref() {
        Some(header) => header,
        None => return Err(BlockHeaderError::MissingHeader),
    };

    let block_header_roots: BlockHeaderRoots = block_header.clone().try_into()?;

    if header_roots != block_header_roots {
        return Err(BlockHeaderError::MismatchedRoots(Box::new((
            header_roots,
            block_header_roots,
        ))));
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub(crate) struct HeaderRecordWithNumber {
    pub block_hash: Vec<u8>,
    pub total_difficulty: Vec<u8>,
    pub block_number: u64,
}

impl TryFrom<Block> for HeaderRecordWithNumber {
    type Error = BlockHeaderError;
    fn try_from(block: Block) -> Result<Self, Self::Error> {
        let block_header = match block.header.clone() {
            Some(header) => header,
            None => {
                return Err(BlockHeaderError::MissingHeader);
            }
        };
        let header_record_with_number = HeaderRecordWithNumber {
            block_hash: block.hash.clone(),
            total_difficulty: block_header
                .total_difficulty
                .as_ref()
                .ok_or(BlockHeaderError::InvalidTotalDifficulty)?
                .bytes
                .clone(),
            block_number: block.number,
        };
        Ok(header_record_with_number)
    }
}
