pub mod error;

use crate::receipts::error::ReceiptError;
use firehose_protos::ethereum_v2::Block;
use revm_primitives::hex;

/// Verifies the receipt root in a given block's header against a
/// computed receipt root from the block's body.
///
/// # Arguments
///
/// * `block` reference to the block which the root will be verified  
pub fn check_receipt_root(block: &Block) -> Result<(), ReceiptError> {
    let computed_root = block.calculate_receipt_root()?;
    let receipt_root = match block.header {
        Some(ref header) => header.receipt_root.as_slice(),
        None => return Err(ReceiptError::MissingRoot),
    };
    if computed_root.as_slice() != receipt_root {
        return Err(ReceiptError::MismatchedRoot(
            hex::encode(computed_root.as_slice()),
            hex::encode(receipt_root),
        ));
    }

    Ok(())
}
