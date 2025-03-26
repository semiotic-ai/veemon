use crate::traits::EraValidationContext;
use merkle_proof::MerkleTree;
use primitive_types::H256;

const SOLANA_EPOCH_LENGTH: usize = 432_000;
const SOLANA_HISTORICAL_TREE_DEPTH: usize = 19;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolanaHistoricalRoots(pub Vec<H256>);

pub struct SolanaValidator {
    pub historical_roots: SolanaHistoricalRoots,
}

impl SolanaValidator {
    /// Creates a new Solana validator.
    pub fn new(historical_roots: SolanaHistoricalRoots) -> Self {
        Self { historical_roots }
    }

    /// Validates the era using the historical roots.
    pub fn validate_era(&self, input: (usize, Vec<H256>)) -> Result<(), String> {
        self.historical_roots.validate_era(input)
    }
}

impl EraValidationContext for SolanaHistoricalRoots {
    type EraInput = (usize, Vec<H256>);
    type EraOutput = Result<(), String>;

    fn validate_era(&self, input: Self::EraInput) -> Self::EraOutput {
        let era_number = input.0;
        let block_roots = input.1;
        if block_roots.len() != SOLANA_EPOCH_LENGTH {
            return Err(format!(
                "Number of execution block hashes must match the epoch length: expected {}, got {}",
                SOLANA_EPOCH_LENGTH,
                block_roots.len()
            ));
        }

        let root = MerkleTree::create(block_roots.as_slice(), SOLANA_HISTORICAL_TREE_DEPTH).hash();

        // Check that root matches the expected historical root
        if root != self.0[era_number] {
            return Err(format!(
                "Historical root mismatch: expected {:?}, got {:?}",
                self.0[era_number], root
            ));
        }
        Ok(())
    }
}
