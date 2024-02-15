pub mod error;
pub mod logs;
pub mod receipt;

// use crate::protos::block::Block;
use crate::receipts::error::ReceiptError;
use crate::receipts::receipt::FullReceipt;
use crate::sf::ethereum::r#type::v2::Block;
use reth_primitives::bytes::BufMut;
use reth_primitives::hex;
use reth_primitives::proofs::ordered_trie_root_with_encoder;
use reth_rlp::Encodable;
use revm_primitives::B256;

const BYZANTINUM_FORK_BLOCK: u64 = 4_370_000;

/// Verifies the receipt root in a given block's header against a
/// computed receipt root from the block's body.
///
/// # Arguments
///
/// * `block` reference to the block which the root will be verified  
pub fn check_receipt_root(block: &Block) -> Result<(), ReceiptError> {
    let computed_root = calc_receipt_root(block)?;
    let receipt_root = match block.header {
        Some(ref header) => header.receipt_root.as_slice(),
        None => return Err(ReceiptError::MissingRoot),
    };
    if computed_root.as_bytes() != receipt_root {
        return Err(ReceiptError::MismatchedRoot(
            hex::encode(computed_root.as_bytes()),
            hex::encode(receipt_root),
        ));
    }

    Ok(())
}

/// Calculates the trie receipt root of a given block recepits
///
/// It uses the traces to aggregate receipts from blocks and then
/// verifies them against the trie root present in the block header
///
///  # Arguments
///
/// * `block` reference to the block which the root will be verified  
fn calc_receipt_root(block: &Block) -> Result<B256, ReceiptError> {
    let mut receipts = Vec::new();

    for trace in &block.transaction_traces {
        receipts.push(FullReceipt::try_from(trace)?);
    }

    let encoder = get_encoder(block);

    Ok(ordered_trie_root_with_encoder(&receipts, encoder))
}

/// Encodes full rceipts using [RLP serialization](https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp)
///
/// For blocks before the Byzantium fork, it uses a specific RLP encoding that includes the receipt's header length values, state root,
/// cumulative gas used, bloom filter, and logs.
/// For blocks at or after the Byzantium fork, it encodes the receipt's inner contents without the header.
///
/// This function is useful for computing the trie root hash which in reth needs to be rlp encoded.
///
/// # Arguments
///
/// * `block` reference to the [`Block`] where [`FullReceipt`] will be extracted from
///
///  # Returns
///
/// a function that takes a refenrece to a [`FullReceipt`],
/// and a mutable reference to a type implementing the [`BufMut`].
/// All the data from the receipts in written into the `BufMut` buffer

fn get_encoder(block: &Block) -> fn(&FullReceipt, &mut dyn BufMut) {
    if block.number >= BYZANTINUM_FORK_BLOCK {
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

/// Encodes receipt header using [RLP serialization](https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp)
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
