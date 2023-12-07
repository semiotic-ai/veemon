pub mod error;
pub mod logs;
pub mod receipt;

use crate::protos::block::Block;
use crate::receipts::error::ReceiptError;
use crate::receipts::receipt::FullReceipt;
use reth_primitives::bytes::BufMut;
use reth_primitives::hex;
use reth_primitives::proofs::ordered_trie_root_with_encoder;
use reth_rlp::Encodable;
use revm_primitives::B256;

pub fn check_receipt_root(block: &Block) -> Result<(), ReceiptError> {
    let computed_root = calc_receipt_root(block)?;

    if computed_root.as_bytes() != block.header.receipt_root.as_slice() {
        return Err(ReceiptError::MismatchedRoot(
            hex::encode(computed_root.as_bytes()),
            hex::encode(block.header.receipt_root.as_slice()),
        ));
    }

    Ok(())
}

fn calc_receipt_root(block: &Block) -> Result<B256, ReceiptError> {
    let mut receipts = Vec::new();

    for trace in &block.transaction_traces {
        receipts.push(FullReceipt::try_from(trace)?);
    }

    let encoder = get_encoder(block);

    Ok(ordered_trie_root_with_encoder(&receipts, encoder))
}

fn get_encoder(block: &Block) -> fn(&FullReceipt, &mut dyn BufMut) {
    if block.number >= 4_370_000 {
        |r: &FullReceipt, out: &mut dyn BufMut| r.receipt.encode_inner(out, false)
    } else {
        |r: &FullReceipt, out: &mut dyn BufMut| {
            receipt_rlp_header(r).encode(out);
            r.state_root.as_slice().encode(out);
            r.receipt.receipt.cumulative_gas_used.encode(out);
            r.receipt.bloom.encode(out);
            r.receipt.receipt.logs.encode(out);
        }
    }
}

fn receipt_rlp_header(receipt: &FullReceipt) -> reth_rlp::Header {
    let payload_length = receipt.state_root.as_slice().length()
        + receipt.receipt.receipt.cumulative_gas_used.length()
        + receipt.receipt.bloom.length()
        + receipt.receipt.receipt.logs.length();

    reth_rlp::Header {
        list: true,
        payload_length,
    }
}
