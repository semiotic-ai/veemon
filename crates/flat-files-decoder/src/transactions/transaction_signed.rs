use crate::sf::ethereum::r#type::v2::TransactionTrace;
use crate::transactions::error::TransactionError;
use reth_primitives::{Signature, Transaction, TransactionSigned};
use revm_primitives::{hex, B256};
use std::str::FromStr;

impl TryFrom<&TransactionTrace> for TransactionSigned {
    type Error = TransactionError;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        let transaction = Transaction::try_from(trace)?;
        let signature = Signature::try_from(trace)?;
        let hash = B256::from_str(&hex::encode(trace.hash.as_slice()))
            .map_err(|_| TransactionError::MissingCall)?;
        let tx_signed = TransactionSigned {
            transaction,
            signature,
            hash,
        };
        Ok(tx_signed)
    }
}
