use protobuf::EnumOrUnknown;
use reth_primitives::{Log, Receipt};
use crate::protos::block::{TransactionTrace, TransactionTraceStatus};
use crate::receipts::error::ReceiptError;
use crate::receipts::logs::map_logs;
use crate::transactions::tx_type::map_tx_type;

impl TryFrom<&TransactionTrace> for Receipt {
    type Error = ReceiptError;

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

fn map_success(status: &EnumOrUnknown<TransactionTraceStatus>) -> Result<bool, ReceiptError> {
    let status = status.enum_value().map_err(|_| ReceiptError::InvalidStatus)?;
    Ok(status == TransactionTraceStatus::SUCCEEDED)
}

