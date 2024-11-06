//! Execution Layer functionality to build a Merkle Patricia Trie (MPT) from Ethereum receipts
//! and generate inclusion proofs for specified receipts within the trie. It includes data structures
//! for parsing and handling receipt data, as well as utilities for encoding and decoding as required
//! by the Ethereum specification.

use alloy_primitives::{Bloom, U256};
use alloy_rlp::Encodable;
use reth_primitives::{Log, Receipt, ReceiptWithBloom, TxType};
use reth_trie_common::{proof::ProofRetainer, root::adjust_index_for_rlp, HashBuilder, Nibbles};
use serde::{Deserialize, Deserializer, Serialize};
use std::vec::IntoIter;

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
        use alloy_primitives::{bytes, fixed_bytes, Address};
        use rand::{self, rngs::OsRng, RngCore};

        fn fake_log() -> Log {
            // generate random slice of bytes
            let mut random_bytes = [0u8; 20];
            OsRng.fill_bytes(&mut random_bytes);
            // Create a 32-byte array initialized with zeros
            let mut bytes = [0u8; 32];

            // Insert the random bytes into the last 20 bytes of the array
            bytes[12..].copy_from_slice(&random_bytes);

            // Generate a static Log based on an actual log receipt
            Log::new_unchecked(
                Address::random(),
                vec![
                    fixed_bytes!(
                        "e1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c"
                    ),
                    fixed_bytes!(
                        "0000000000000000000000003328f7f4a1d1c57c35df56bbf0c9dcafca309c49"
                    ),
                ],
                bytes!("0000000000000000000000000000000000000000000000000dcf09da3e1eb9f3"),
            )
        }

        let logs: Vec<Log> = (0..5).map(|_| fake_log()).collect();

        ReceiptJson {
            tx_type: TxType::Eip1559, // Replace with any desired variant
            block_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                .to_string(),
            block_number: "0x1a".to_string(),
            logs,
            cumulative_gas_used: U256::from(0x5208),
            status: true,
            logs_bloom: Bloom::default(),
        }
    }
}

/// Represents a leaf in the trie for which a proof is to be generated, i.e., the target of the proof.
/// The `nibbles` represent the path to the leaf in the trie, and the `value` is the data stored at the leaf.
#[derive(Debug)]
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

pub struct TargetLeaves(Vec<TargetLeaf>);

impl TargetLeaves {
    fn new() -> Self {
        TargetLeaves(Vec::new())
    }

    pub fn from_indices(
        target_idxs: &[usize],
        receipts: &[ReceiptWithBloom],
    ) -> Result<Self, &'static str> {
        let mut index_buffer = Vec::new();
        let mut value_buffer = Vec::new();
        let mut targets = TargetLeaves::new();
        let receipts_len = receipts.len();

        for &target_idx in target_idxs {
            if target_idx >= receipts_len {
                return Err("Index out of bounds");
            }

            index_buffer.clear();
            value_buffer.clear();

            // Adjust the index and encode it
            let index = adjust_index_for_rlp(target_idx, receipts_len);
            index.encode(&mut index_buffer);

            // Generate nibble path from the index buffer
            let nibble = Nibbles::unpack(&index_buffer);

            // Encode the receipt and create TargetLeaf
            receipts[index].encode_inner(&mut value_buffer, false);
            targets
                .0
                .push(TargetLeaf::new(nibble, value_buffer.clone()));
        }

        Ok(targets)
    }
}

impl IntoIterator for TargetLeaves {
    type Item = TargetLeaf;
    type IntoIter = IntoIter<TargetLeaf>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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

/// builds the trie to generate proofs from the Receipts
/// generate a different root. Make sure that the source of receipts sorts them by `logIndex`
/// # Example
///
/// ```no_run
/// # use reth_primitives::ReceiptWithBloom;
/// # use forrestrie::execution_layer::build_trie_with_proofs;
/// # let receipts_json = vec![];
/// // Assume `receipts_json` is a vector of deserialized receipt objects from an execution block given by an Ethereum node.
/// let receipts_with_bloom: Vec<ReceiptWithBloom> = receipts_json
///     .iter()
///     .map(ReceiptWithBloom::try_from)
///     .collect::<Result<_, _>>().unwrap();
///
/// // Specify the indices of receipts for which proofs are needed.
/// let target_indices = &[0, 2, 5];
///
/// // Build the trie and obtain proofs.
/// let mut hash_builder = build_trie_with_proofs(&receipts_with_bloom, target_indices);
///
/// // Retrieve the root hash of the trie, and retain the proofs so they can be verified.
/// let trie_root = hash_builder.root();
/// ```
pub fn build_trie_with_proofs(receipts: &[ReceiptWithBloom], target_idxs: &[usize]) -> HashBuilder {
    // Initialize ProofRetainer with the target nibbles (the keys for which we want proofs)
    let receipts_len = receipts.len();
    let targets: Vec<Nibbles> = target_idxs
        .iter()
        .map(|&i| {
            let index = adjust_index_for_rlp(i, receipts_len);
            let mut index_buffer = Vec::new();
            index.encode(&mut index_buffer);
            Nibbles::unpack(&index_buffer)
        })
        .collect();

    let proof_retainer = ProofRetainer::new(targets);
    let mut hb = HashBuilder::default().with_proof_retainer(proof_retainer);

    for i in 0..receipts_len {
        // Adjust the index for RLP
        let index = adjust_index_for_rlp(i, receipts_len);

        // Encode the index into nibbles
        let mut index_buffer = Vec::new();
        index.encode(&mut index_buffer);
        let index_nibbles = Nibbles::unpack(&index_buffer);

        // Encode the receipt value
        let mut value_buffer = Vec::new();
        receipts[index].encode_inner(&mut value_buffer, false);

        hb.add_leaf(index_nibbles, &value_buffer);
    }

    hb
}

#[cfg(test)]
mod tests {
    use super::*;
    use reth_trie_common::proof::verify_proof;

    #[test]
    fn test_compute_receipts_trie_root_and_proof() {
        let block_receipts: ReceiptsFromBlock = (0_i32..10).map(|_| ReceiptJson::fake()).collect();

        let receipts_with_bloom: Result<Vec<ReceiptWithBloom>, String> = block_receipts
            .result
            .iter()
            .map(ReceiptWithBloom::try_from)
            .collect::<Result<Vec<_>, _>>();

        // computes the root and verify against existing data
        let mut hb: HashBuilder;
        //target_idxs are the logIndexes for receipts to get proofs from.
        // these values are arbitrary
        let target_idxs = &[4];
        let targets: TargetLeaves;

        match receipts_with_bloom {
            Ok(receipts) => {
                hb = build_trie_with_proofs(&receipts, target_idxs);

                // build some of the targets to get proofs for them
                targets = TargetLeaves::from_indices(target_idxs, &receipts).unwrap();
            }
            Err(e) => {
                // Handle the error (e.g., by logging or panicking)
                panic!("Failed to convert receipts: {}", e);
            }
        }

        // necessary to call this method to retain proofs
        hb.root();

        // verifies proof for retained targets
        let proof = hb.take_proof_nodes();
        for target in targets {
            assert_eq!(
                verify_proof(
                    hb.root(),
                    target.nibbles.clone(),
                    Some(target.value.to_vec()),
                    proof
                        .clone()
                        .matching_nodes_sorted(&target.nibbles)
                        .iter()
                        .map(|(_, node)| node)
                ),
                Ok(())
            );
        }
    }
}
