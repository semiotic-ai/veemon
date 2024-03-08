// use crate::protos::block::transaction_trace::Type;
use reth_primitives::TxType;
use sf_protos::ethereum::r#type::v2::transaction_trace::Type;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransactionTypeError {
    #[error("Transaction type is missing")]
    Missing,
}

fn tx_to_reth_tx(tx_type: Type) -> TxType {
    match tx_type {
        Type::TrxTypeLegacy => TxType::Legacy,
        Type::TrxTypeAccessList => TxType::EIP2930,
        Type::TrxTypeDynamicFee => TxType::EIP1559,
    }
}

pub fn map_tx_type(tx_type: &i32) -> Result<TxType, TransactionTypeError> {
    let tx_type = Type::try_from(*tx_type).map_err(|_| TransactionTypeError::Missing)?; // 1
    let tx_type = tx_to_reth_tx(tx_type);
    Ok(tx_type)
}
