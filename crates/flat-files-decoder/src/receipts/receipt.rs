use protobuf::EnumOrUnknown;
use reth_primitives::{Log, Receipt, TxType};
use crate::protos::block::{TransactionTrace, TransactionTraceStatus};
use crate::protos::block::transaction_trace::Type;
use crate::receipts::error::InvalidReceiptError;
use crate::receipts::logs::map_logs;

pub(crate) fn map_receipt_from_trace(trace: &TransactionTrace) -> Result<Receipt, InvalidReceiptError> {
    let success = map_success(&trace.status)?;

    let tx_type = map_tx_type(&trace.type_)?;

    let logs: Vec<Log> = map_logs(&trace.receipt.logs)?;

    let cumulative_gas_used = trace.receipt.cumulative_gas_used;

    Ok(Receipt {
        tx_type,
        success,
        cumulative_gas_used,
        logs,
    })
}

fn map_success(status: &EnumOrUnknown<TransactionTraceStatus>) -> Result<bool, InvalidReceiptError> {
    let status = status.enum_value().map_err(|_| InvalidReceiptError::Status)?;
    Ok(status == TransactionTraceStatus::SUCCEEDED)
}

// TODO: check -> nothing maps to TxType::EIP4844
fn map_tx_type(tx_type: &EnumOrUnknown<Type>) -> Result<TxType, InvalidReceiptError> {
    let tx_type = tx_type.enum_value().map_err(|_| InvalidReceiptError::TxType)?;
    match tx_type {
        Type::TRX_TYPE_LEGACY => Ok(TxType::Legacy),
        Type::TRX_TYPE_ACCESS_LIST => Ok(TxType::EIP2930),
        Type::TRX_TYPE_DYNAMIC_FEE => Ok(TxType::EIP1559),
    }
}