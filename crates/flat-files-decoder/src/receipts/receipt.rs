use crate::protos::block::{TransactionTrace, TransactionTraceStatus};
use crate::receipts::error::ReceiptError;
use crate::receipts::logs::map_logs;
use crate::transactions::tx_type::map_tx_type;
use protobuf::EnumOrUnknown;
use reth_primitives::{hex, Bloom, Log, Receipt, ReceiptWithBloom};

pub struct FullReceipt {
    pub receipt: ReceiptWithBloom,
    pub state_root: Vec<u8>,
}

impl TryFrom<&TransactionTrace> for FullReceipt {
    type Error = ReceiptError;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        let success = map_success(&trace.status)?;
        let tx_type = map_tx_type(&trace.type_)?;
        let logs: Vec<Log> = map_logs(&trace.receipt.logs)?;
        let cumulative_gas_used = trace.receipt.cumulative_gas_used;

        let receipt = Receipt {
            success,
            tx_type,
            logs,
            cumulative_gas_used,
        };

        let bloom = map_bloom(&trace.receipt.logs_bloom)?;

        let receipt = ReceiptWithBloom { receipt, bloom };

        let state_root = trace.receipt.state_root.clone();

        Ok(Self {
            receipt,
            state_root,
        })
    }
}

fn map_success(status: &EnumOrUnknown<TransactionTraceStatus>) -> Result<bool, ReceiptError> {
    let status = status
        .enum_value()
        .map_err(|_| ReceiptError::InvalidStatus)?;
    Ok(status == TransactionTraceStatus::SUCCEEDED)
}

fn map_bloom(slice: &[u8]) -> Result<Bloom, ReceiptError> {
    if slice.len() == 256 {
        let array: [u8; 256] = slice
            .try_into()
            .expect("Slice length doesn't match array length");
        Ok(Bloom(array))
    } else {
        Err(ReceiptError::InvalidBloom(hex::encode(slice)))
    }
}
