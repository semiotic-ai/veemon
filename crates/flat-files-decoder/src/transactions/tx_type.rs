// use crate::protos::block::transaction_trace::Type;
use crate::sf::ethereum::r#type::v2::transaction_trace::Type;
use reth_primitives::TxType;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransactionTypeError {
    #[error("Transaction type is missing")]
    Missing,
}

impl From<Type> for TxType {
    fn from(tx_type: Type) -> Self {
        match tx_type {
            Type::TrxTypeLegacy => Self::Legacy,
            Type::TrxTypeAccessList => Self::EIP2930,
            Type::TrxTypeDynamicFee => Self::EIP1559,
        }
    }
}

pub fn map_tx_type(tx_type: &i32) -> Result<TxType, TransactionTypeError> {
    let tx_type = Type::try_from(*tx_type).map_err(|_| TransactionTypeError::Missing)?; // 1
    let tx_type = TxType::from(tx_type);
    Ok(tx_type)
}
