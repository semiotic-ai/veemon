use crate::receipts::error::ReceiptError;
use crate::receipts::logs::map_logs;
use crate::transactions::tx_type::map_tx_type;
use alloy_primitives::FixedBytes;
use reth_primitives::{hex, Bloom, Log, Receipt, ReceiptWithBloom};
use sf_protos::ethereum::r#type::v2::TransactionTrace;

pub struct FullReceipt {
    pub receipt: ReceiptWithBloom,
    pub state_root: Vec<u8>,
}

impl TryFrom<&TransactionTrace> for FullReceipt {
    type Error = ReceiptError;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        let success = map_success(&trace.status)?;
        let tx_type = map_tx_type(&trace.r#type)?;
        let trace_receipt = match &trace.receipt {
            Some(receipt) => receipt,
            None => return Err(ReceiptError::MissingReceipt),
        };
        let logs: Vec<Log> = map_logs(&trace_receipt.logs)?;
        let cumulative_gas_used = trace_receipt.cumulative_gas_used;

        let receipt = Receipt {
            success,
            tx_type,
            logs,
            cumulative_gas_used,
        };

        let bloom = map_bloom(&trace_receipt.logs_bloom)?;

        let receipt = ReceiptWithBloom { receipt, bloom };

        let state_root = &trace_receipt.state_root;

        Ok(Self {
            receipt,
            state_root: state_root.to_vec(),
        })
    }
}

fn map_success(status: &i32) -> Result<bool, ReceiptError> {
    Ok(*status == 1)
}

fn map_bloom(slice: &[u8]) -> Result<Bloom, ReceiptError> {
    if slice.len() == 256 {
        let array: [u8; 256] = slice
            .try_into()
            .expect("Slice length doesn't match array length");
        Ok(Bloom(FixedBytes(array)))
    } else {
        Err(ReceiptError::InvalidBloom(hex::encode(slice)))
    }
}
