
use crate::traits::EraValidationContext;
use types::{historical_summary::HistoricalSummary, BeaconBlock};
use merkle_proof::MerkleTree;

pub struct EthereumPostCapellaValidator {
    pub historical_summary: HistoricalSummary,
}

impl EthereumPostCapellaValidator {
    /// Creates a new Ethereum post-merge validator.
    pub fn new(historical_summary: HistoricalSummary) -> Self {
        Self { historical_summary }
    }

    /// Validates the era using the historical summary.
    pub fn validate_era(&self, block: &BeaconBlock) -> Result<(), String> {
        self.historical_summary.validate_era(block)
    }
}

impl EraValidationContext for HistoricalSummary {
    type EraInput = Vec<BeaconBlock>;
    type EraOutput = Result<(), String>;

    fn validate_era(&self, input: Self::EraInput) -> Self::EraOutput {
       // Get era number from the slot of the first block: era = slot / 8192. Return an error if
        // not an even multiple of 8192.
        let era = input[0].slot / 8192;
        if input[0].slot % 8192 != 0 {
            return Err(format!("Invalid era number: {}", era));
        }

        // Check that there are 8192 blocks in the era.
        if input.len() != 8192 {
            return Err(format!(
                "Invalid number of blocks in era {}: expected 8192, got {}",
                era,
                input.len()
            ));
        }

        // Calculate the beacon block roots for each beacon block in the era.
        let mut roots = Vec::new();
        for block in &input {
            let root = block.tree_hash_root();
            roots.push(root);
        }

        // Calculate the tree hash root of the beacon block roots and compare against the
        // historical_summary.block_summary_root for the era.
        let beacon_block_roots_tree_hash_root = MerkleTree::create(&roots, 13).hash();

        let true_root = self
            .historical_summary
            .get_block_summary_root(era)
            .ok_or_else(|| format!("No block summary root for era {}", era))?;

        if beacon_block_roots_tree_hash_root != true_root {
            return Err(format!(
                "Invalid block summary root for era {}: expected {}, got {}",
                era,
                true_root,
                beacon_block_roots_tree_hash_root
            ));
        }
        

        Ok(())
    }
}

