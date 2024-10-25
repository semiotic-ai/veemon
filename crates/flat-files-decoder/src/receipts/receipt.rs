use crate::receipts::error::ReceiptError;
use alloy_primitives::{Bloom, FixedBytes};
use firehose_protos::ethereum_v2::TransactionTrace;
use reth_primitives::{Log, Receipt, ReceiptWithBloom};
use revm_primitives::hex;

pub(crate) struct FullReceipt {
    pub receipt: ReceiptWithBloom,
    pub state_root: Vec<u8>,
}

impl TryFrom<&TransactionTrace> for FullReceipt {
    type Error = ReceiptError;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        let success = trace.is_success();
        let tx_type = trace.try_into()?;

        let trace_receipt = match &trace.receipt {
            Some(receipt) => receipt,
            None => return Err(ReceiptError::MissingReceipt),
        };
        let logs = trace_receipt
            .logs
            .iter()
            .map(Log::try_from)
            .collect::<Result<Vec<_>, _>>()?;

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

fn map_bloom(slice: &[u8]) -> Result<Bloom, ReceiptError> {
    if slice.len() == 256 {
        let array: [u8; 256] = slice.try_into()?;
        Ok(Bloom(FixedBytes(array)))
    } else {
        Err(ReceiptError::InvalidBloom(hex::encode(slice)))
    }
}
