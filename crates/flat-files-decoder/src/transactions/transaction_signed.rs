use crate::transactions::error::TransactionError;
use reth_primitives::TransactionSigned;
use revm_primitives::{hex, B256};
use sf_protos::ethereum::r#type::v2::TransactionTrace;
use std::str::FromStr;

use super::{signature::signature_from_trace, transaction::trace_to_transaction};

pub fn trace_to_signed(trace: &TransactionTrace) -> Result<TransactionSigned, TransactionError> {
    let transaction = trace_to_transaction(trace)?;
    let signature = signature_from_trace(trace)?;
    let hash = B256::from_str(&hex::encode(trace.hash.as_slice()))
        .map_err(|_| TransactionError::MissingCall)?;
    let tx_signed = TransactionSigned {
        transaction,
        signature,
        hash,
    };
    Ok(tx_signed)
}
