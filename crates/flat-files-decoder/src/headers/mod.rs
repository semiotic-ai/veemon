pub mod error;

use crate::headers::error::BlockHeaderError;
use crate::protos::block::{Block, BlockHeader};
use protobuf::MessageField;
use reth_primitives::H256;
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeaderRoots {
    pub receipt_root: H256,
    pub transactions_root: H256,
}

impl TryFrom<MessageField<BlockHeader>> for BlockHeaderRoots {
    type Error = BlockHeaderError;

    fn try_from(header: MessageField<BlockHeader>) -> Result<Self, Self::Error> {
        let header = header.unwrap(); // TODO: ERRRORS
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

    let block_header_roots: BlockHeaderRoots = block.header.clone().try_into()?; // TODO: Errors

    if header_roots != block_header_roots {
        return Err(BlockHeaderError::MismatchedRoots(
            header_roots,
            block_header_roots,
        ));
    }

    Ok(())
}
