use crate::protos::block::transaction_trace::Type;
use protobuf::EnumOrUnknown;
use reth_primitives::TxType;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransactionTypeError {
    #[error("Transaction type is missing")]
    Missing,
}

impl From<Type> for TxType {
    // TODO: check -> nothing maps to TxType::EIP4844
    fn from(tx_type: Type) -> Self {
        match tx_type {
            Type::TRX_TYPE_LEGACY => Self::Legacy,
            Type::TRX_TYPE_ACCESS_LIST => Self::EIP2930,
            Type::TRX_TYPE_DYNAMIC_FEE => Self::EIP1559,
        }
    }
}

pub fn map_tx_type(tx_type: &EnumOrUnknown<Type>) -> Result<TxType, TransactionTypeError> {
    let tx_type = tx_type
        .enum_value()
        .map_err(|_| TransactionTypeError::Missing)?;
    Ok(tx_type.into())
}
