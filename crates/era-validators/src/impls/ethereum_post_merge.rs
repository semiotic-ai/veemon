use crate::{impls::common::*, traits::EraValidationContext};
use merkle_proof::MerkleTree;
use primitive_types::H256;
use types::{BeaconBlock, MainnetEthSpec};

pub struct EthereumPostMergeValidator {
    pub historical_roots: Vec<H256>,
}

impl EthereumPostMergeValidator {
    /// Creates a new Ethereum post-merge validator.
    pub fn new(historical_roots: Vec<H256>) -> Self {
        Self { historical_roots }
    }

    /// Validates the era using the historical summary.
    pub fn validate_era(
        &self,
        input: (Vec<Option<H256>>, Vec<BeaconBlock<MainnetEthSpec>>),
    ) -> Result<(), String> {
        self.historical_roots.validate_era(input)
    }
}

impl EraValidationContext for Vec<H256>{
    type EraInput = (Vec<Option<H256>>, Vec<BeaconBlock<MainnetEthSpec>>);
    type EraOutput = Result<(), String>;

    fn validate_era(&self, input: Self::EraInput) -> Self::EraOutput {
        let exec_hashes = input.0;
        let blocks = input.1;

        if blocks.len() != exec_hashes.len() {
            return Err(
                "Number of execution block hashes must match the number of beacon blocks".into(),
            );
        }

        for (block, expected_exec_hash) in blocks.iter().zip(exec_hashes.iter()) {
            // Assuming each beacon block has a method like `execution_payload()`
            // that returns an Option<&ExecutionPayload>
            match get_execution_payload_block_hash(block) {
                Some(execution_block_hash) => {
                    // Compare the block hash from the execution payload to the provided hash.
                    let actual_hash = Some(execution_block_hash);
                    if Some(actual_hash) != Some(*expected_exec_hash) {
                        return Err(format!(
                            "Execution block hash mismatch: expected {:?}, got {:?}",
                            expected_exec_hash, actual_hash
                        ));
                    }
                }
                None => {
                    // If there's no execution payload, make sure no hash was provided.
                    if expected_exec_hash.is_some() {
                        return Err("Unexpected execution block hash for a block without an execution payload".into());
                    }
                }
            }
        }

        // Get era number from the slot of the first block: era = slot / 8192. Return an error if
        // not an even multiple of 8192.
        let era = blocks[0].slot() / 8192;
        if blocks[0].slot() % 8192 != 0 {
            return Err(format!("Invalid era number: {}", era));
        }

        // Calculate the beacon block roots for each beacon block in the era.
        let mut roots = Vec::new();
        for block in &blocks {
            let root = compute_tree_hash_root(block);
            roots.push(root);
        }

        // Calculate the tree hash root of the beacon block roots and compare against the
        // historical_summary.block_summary_root for the era.
        let beacon_block_roots_tree_hash_root = MerkleTree::create(roots.as_slice(), 13).hash();

        let true_root = self[usize::from(era)];

        if beacon_block_roots_tree_hash_root != true_root {
            return Err(format!(
                "Invalid block summary root for era {}: expected {}, got {}",
                era, true_root, beacon_block_roots_tree_hash_root
            ));
        }

        Ok(())
    }
}
