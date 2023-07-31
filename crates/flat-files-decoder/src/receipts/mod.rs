mod logs;
mod receipt;

use std::any::Any;
use anyhow::anyhow;

use reth_blockchain_tree::post_state::PostState;
use crate::protos::block::{Block};
use crate::receipts::receipt::map_receipt_from_trace;

pub fn check_valid_root(block: &Block) -> anyhow::Result<()> {
    let mut post_state = PostState::new();

    // TODO: Extract to separate functions / modules
    for trace in &block.transaction_traces {
        post_state.add_receipt(block.number, map_receipt_from_trace(trace)?);
    }

    let computed_root = post_state.receipts_root(block.number);

    if computed_root.as_bytes() != block.header.receipt_root.as_slice() {
        return Err(anyhow!("Invalid receipt root: {:?} != {:?}", computed_root, block.header.receipt_root));
    }

    return Ok(());
}




