use crate::transactions::error::TransactionError;
use alloy_primitives::FixedBytes;
use firehose_protos::ethereum_v2::TransactionTrace;
use reth_primitives::TransactionSigned;
use revm_primitives::hex;
use std::str::FromStr;

use super::{signature::signature_from_trace, transaction::trace_to_transaction};

pub fn trace_to_signed(trace: &TransactionTrace) -> Result<TransactionSigned, TransactionError> {
    let transaction = trace_to_transaction(trace)?;
    let signature = signature_from_trace(trace)?;
    let hash = FixedBytes::from_str(&hex::encode(trace.hash.as_slice()))
        .map_err(|_| TransactionError::MissingCall)?;
    let tx_signed = TransactionSigned {
        transaction,
        signature,
        hash,
    };
    Ok(tx_signed)
}
