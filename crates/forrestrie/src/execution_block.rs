use ethers::prelude::*;
use reth::primitives::{Log, TxType};
use serde::{Deserialize, Deserializer, Serialize};

// reth::primitives::proofs::calculate_receipt_root;

#[derive(Debug, Deserialize, Serialize)]
pub struct ReceiptWrapper {
    #[serde(rename = "type")]
    #[serde(deserialize_with = "str_to_type")]
    pub tx_type: TxType,
    #[serde(rename = "blockHash")]
    pub block_hash: String,
    #[serde(rename = "blockNumber")]
    pub block_number: String,
    pub logs: Vec<Log>,
    #[serde(rename = "cumulativeGasUsed")]
    pub cumulative_gas_used: U256,
    #[serde(deserialize_with = "status_to_bool")]
    pub status: bool,
}

fn status_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let status_str: &str = Deserialize::deserialize(deserializer)?;
    match status_str {
        "0x1" => Ok(true),
        "0x0" => Ok(false),
        _ => Err(serde::de::Error::custom("Invalid status value")),
    }
}

// Custom deserialization function for TxType
fn str_to_type<'de, D>(deserializer: D) -> Result<TxType, D::Error>
where
    D: Deserializer<'de>,
{
    let tx_type_str: &str = Deserialize::deserialize(deserializer)?;
    // Convert the hex string (without the "0x" prefix) to u8
    let tx_type_value = u8::from_str_radix(tx_type_str.trim_start_matches("0x"), 16)
        .map_err(|_| serde::de::Error::custom("Invalid tx_type value"))?;
    TxType::try_from(tx_type_value).map_err(|_| serde::de::Error::custom("Invalid tx_type value"))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReceiptsFromBlock {
    pub result: Vec<ReceiptWrapper>,
}

#[cfg(test)]
mod tests {

    use std::cell::LazyCell;

    use crate::beacon_block::BlockWrapper;

    use super::*;

    /// Deneb block JSON file shared among contributors.
    /// The block hash is `0x5dde05ab1da7f768ed3ea2d53c6fa0d79c0c2283e52bb0d00842a4bdbf14c0ab`.
    const DENEB_BLOCK_JSON: &str = include_str!("../../../bb-8786333.json");
    const BLOCK_RECEIPTS_JSON: &str = include_str!("../../../eb-19584570-receipts.json");

    const BLOCK_WRAPPER: LazyCell<BlockWrapper> = LazyCell::new(|| {
        serde_json::from_str(DENEB_BLOCK_JSON).expect(
            "For this spike we are using a Deneb block JSON file that has been shared among contributors",
        )
    });

    const RECEIPTS: LazyCell<ReceiptsFromBlock> = LazyCell::new(|| {
        serde_json::from_str(BLOCK_RECEIPTS_JSON).expect(
            "This is all the receipt data from a block, fetch with eth_getBlockReceipts method",
        )
    });

    #[test]
    fn test_parse_wrapped_receipt_into_reth_receipt() {
        let block_wrapper: &LazyCell<BlockWrapper> = &BLOCK_WRAPPER;
        let block = &block_wrapper.data.message;

        let block_body = block.body_deneb().unwrap();
        let payload = &block_body.execution_payload;
        let receits_root = payload.execution_payload.receipts_root;

        let receipts = &RECEIPTS;
    }
}
