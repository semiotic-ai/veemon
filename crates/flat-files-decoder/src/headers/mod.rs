pub mod error;

use crate::headers::error::BlockHeaderError;
use reth_primitives::B256;
use serde::{Deserialize, Serialize};
use sf_protos::ethereum::r#type::v2::{Block, BlockHeader};
use std::fs::File;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeaderRoots {
    pub receipt_root: B256,
    pub transactions_root: B256,
}

impl TryFrom<BlockHeader> for BlockHeaderRoots {
    type Error = BlockHeaderError;

    fn try_from(header: BlockHeader) -> Result<Self, Self::Error> {
        let receipt_root: [u8; 32] = header.receipt_root.as_slice().try_into().unwrap();
        let transactions_root: [u8; 32] = header.transactions_root.as_slice().try_into().unwrap();

        Ok(Self {
            receipt_root: receipt_root.into(),
            transactions_root: transactions_root.into(),
        })
    }
}

pub fn check_valid_header(block: &Block, header_dir: &str) -> Result<(), BlockHeaderError> {
    let header_file_path = format!("{}/{}.json", header_dir, block.number);
    let header_file = File::open(header_file_path)?;

    let header_roots: BlockHeaderRoots = serde_json::from_reader(header_file)?; // TODO: Errors

    let block_header = match block.header.as_ref() {
        Some(header) => header,
        None => return Err(BlockHeaderError::MissingHeader),
    };
    let block_header_roots: BlockHeaderRoots = block_header.clone().try_into()?; // TODO: Errors

    if header_roots != block_header_roots {
        return Err(BlockHeaderError::MismatchedRoots(Box::new((
            header_roots,
            block_header_roots,
        ))));
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct HeaderRecordWithNumber {
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
