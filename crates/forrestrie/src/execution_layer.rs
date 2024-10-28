use alloy_primitives::{Bloom, U256};
use alloy_rlp::Encodable;
use reth_primitives::{Log, Receipt, ReceiptWithBloom, TxType};
use reth_trie_common::{proof::ProofRetainer, root::adjust_index_for_rlp, HashBuilder, Nibbles};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ReceiptJson {
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
    // TODO: should we trust logsBloom provided or calculate it from the logs?
    #[serde(rename = "logsBloom")]
    pub logs_bloom: Bloom,
}

impl ReceiptJson {
    #[cfg(test)]
    fn fake() -> Self {
        use alloy_primitives::{Address, Bytes};
        use rand::{self, Rng};

        fn fake_log() -> Log {
            // generate random slice of bytes
            let mut rng = rand::thread_rng();

            // Generate a random u32
            let random_u32: u32 = rng.gen();

            Log::new(
                Address::default(),
                vec![],
                Bytes::from(random_u32.to_be_bytes().to_vec()),
            )
            .unwrap()
        }

        let logs: Vec<Log> = (3..5).into_iter().map(|_| fake_log()).collect();
        dbg!(&logs);

        ReceiptJson {
            tx_type: TxType::Eip1559, // Replace with any desired variant
            block_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                .to_string(),
            block_number: "0x1a".to_string(),
            logs,
            cumulative_gas_used: U256::from(0x5208), // Mock gas used value
            status: true,                            // Mock status as successful
            logs_bloom: Bloom::default(),            // Mock an empty logs bloom
        }
    }
}

/// Represents a leaf in the trie for which a proof is to be generated, i.e., the target of the proof.
/// The `nibbles` represent the path to the leaf in the trie, and the `value` is the data stored at the leaf.
pub struct TargetLeaf {
    pub nibbles: Nibbles,
    pub value: Vec<u8>,
}

impl TargetLeaf {
    // Constructor to create a new TargetLeaf
    pub fn new(nibbles: Nibbles, value: Vec<u8>) -> Self {
        TargetLeaf { nibbles, value }
    }
}

impl TryFrom<&ReceiptJson> for ReceiptWithBloom {
    type Error = String;

    fn try_from(receipt_json: &ReceiptJson) -> Result<Self, Self::Error> {
        let cumulative_gas_used = receipt_json
            .cumulative_gas_used
            .try_into()
            .map_err(|_| "Failed to convert U256 to u64".to_string())?;

        let receipt = Receipt {
            tx_type: receipt_json.tx_type,
            success: receipt_json.status,
            cumulative_gas_used,
            logs: receipt_json.logs.clone(),
            // NOTICE: receipts will have more fields depending of the EVM chain.
            // this is how to handle them in the futuro
            // #[cfg(feature = "optimism")]
            // deposit_nonce: None, // Handle Optimism-specific fields as necessary
            // #[cfg(feature = "optimism")]
            // deposit_receipt_version: None,
        };

        Ok(ReceiptWithBloom {
            bloom: receipt_json.logs_bloom,
            receipt,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReceiptsFromBlock {
    pub result: Vec<ReceiptJson>,
}

impl FromIterator<ReceiptJson> for ReceiptsFromBlock {
    fn from_iter<I: IntoIterator<Item = ReceiptJson>>(iter: I) -> Self {
        ReceiptsFromBlock {
            result: iter.into_iter().collect(),
        }
    }
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

// builds the trie to generate proofs from the Receipts
// generate a different root. Make sure that the source of receipts sorts them by `logIndex`
pub fn build_trie_with_proofs(receipts: &[ReceiptWithBloom], target_idxs: &[usize]) -> HashBuilder {
    let mut index_buffer = Vec::new();
    let mut value_buffer = Vec::new();

    // Initialize ProofRetainer with the target nibbles (the keys for which we want proofs)
    let targets: Vec<Nibbles> = target_idxs
        .iter()
        .map(|&i| {
            let index = adjust_index_for_rlp(i, receipts.len());
            index_buffer.clear();
            index.encode(&mut index_buffer);
            Nibbles::unpack(&index_buffer)
        })
        .collect();

    let proof_retainer: ProofRetainer = ProofRetainer::new(targets);
    let mut hb = HashBuilder::default().with_proof_retainer(proof_retainer);

    let receipts_len = receipts.len();

    for i in 0..receipts_len {
        index_buffer.clear();
        value_buffer.clear();

        let index = adjust_index_for_rlp(i, receipts_len);
        index.encode(&mut index_buffer);

        receipts[index].encode_inner(&mut value_buffer, false);
        // NOTICE: if the ProofRetainer is set, add_leaf automatically retains the proofs for the targets
        hb.add_leaf(Nibbles::unpack(&index_buffer), &value_buffer);
    }

    hb
}

#[cfg(test)]
mod tests {
    use primitive_types::H256;
    use reth_trie_common::proof::verify_proof;

    use std::cell::LazyCell;

    use super::*;

    #[test]
    fn test_compute_receipts_trie_root_and_proof() {
        // TODO: instead of generating receipts, pick a small exempt from
        // the execution block that fits here. It should work better,
        // since faking logs requires the log to be properly rlp encoded
        let block_receipts: ReceiptsFromBlock =
            (0..10).into_iter().map(|_| ReceiptJson::fake()).collect();

        let receipts_with_bloom: Result<Vec<ReceiptWithBloom>, String> = block_receipts
            .result
            .iter()
            .map(ReceiptWithBloom::try_from)
            .collect::<Result<Vec<_>, _>>();

        // computes the root and verify against existing data
        let mut hb: HashBuilder;
        //target_idxs are the logIndexes for receipts to get proofs from.
        // these values are arbitrary
        let target_idxs = &[0, 1, 2];
        let mut targets: Vec<TargetLeaf> = Vec::new();
        let receipts_len;

        match receipts_with_bloom {
            Ok(receipts) => {
                hb = build_trie_with_proofs(&receipts, target_idxs);
                let calculated_root = H256::from(hb.root().0);
                dbg!(&calculated_root);
                // assert_eq!(calculated_root, receipts_root, "Roots do not match!");

                let mut index_buffer = Vec::new();
                let mut value_buffer = Vec::new();

                // build some of the targets to get proofs for them
                receipts_len = receipts.len();
                for i in target_idxs {
                    index_buffer.clear();
                    value_buffer.clear();

                    let index = adjust_index_for_rlp(*i, receipts_len);
                    index.encode(&mut index_buffer);

                    let nibble = Nibbles::unpack(&index_buffer);

                    receipts[index].encode_inner(&mut value_buffer, false);
                    targets.push(TargetLeaf::new(nibble, value_buffer.clone()));
                }
            }
            Err(e) => {
                // Handle the error (e.g., by logging or panicking)
                panic!("Failed to convert receipts: {}", e);
            }
        }

        // verifies proof for retained targets
        let proof = hb.take_proof_nodes();
        for target in targets.iter() {
            let proof1 = proof
                .iter()
                .filter_map(|(k, v)| target.nibbles.starts_with(k).then_some(v));

            assert_eq!(
                verify_proof(
                    hb.root(),
                    target.nibbles.clone(),
                    Some(target.value.to_vec()),
                    proof1.clone(),
                ),
                Ok(())
            );
        }
    }
}
