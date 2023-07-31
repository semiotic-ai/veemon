use protobuf::EnumOrUnknown;
use reth_primitives::{Log, Receipt, TxType};
use crate::protos::block::{TransactionTrace, TransactionTraceStatus};
use crate::protos::block::transaction_trace::Type;
use crate::receipts::error::InvalidReceiptError;
use crate::receipts::logs::map_logs;

impl TryFrom<&TransactionTrace> for Receipt {
    type Error = InvalidReceiptError;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        let success = map_success(&trace.status)?;

        let tx_type = map_tx_type(&trace.type_)?;

        let logs: Vec<Log> = map_logs(&trace.receipt.logs)?;

        let cumulative_gas_used = trace.receipt.cumulative_gas_used;

        Ok(Self {
            tx_type,
            success,
            cumulative_gas_used,
            logs,
        })
    }
}

fn map_success(status: &EnumOrUnknown<TransactionTraceStatus>) -> Result<bool, InvalidReceiptError> {
    let status = status.enum_value().map_err(|_| InvalidReceiptError::Status)?;
    Ok(status == TransactionTraceStatus::SUCCEEDED)
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

fn map_tx_type(tx_type: &EnumOrUnknown<Type>) -> Result<TxType, InvalidReceiptError> {
    let tx_type = tx_type.enum_value().map_err(|_| InvalidReceiptError::TxType)?;
    Ok(tx_type.into())
}