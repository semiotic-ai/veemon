use firehose_protos::ethereum_v2::transaction_trace::Type;
use reth_primitives::TxType;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransactionTypeError {
    #[error("Transaction type is missing")]
    Missing,
}

pub fn map_tx_type(tx_type: &i32) -> Result<TxType, TransactionTypeError> {
    let tx_type = Type::try_from(*tx_type).map_err(|_| TransactionTypeError::Missing)?; // 1
    Ok(TxType::from(tx_type))
}
