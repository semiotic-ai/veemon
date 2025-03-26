use crate::traits::EraValidationContext;
use primitive_types::H256;

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
    pub fn validate_era(
        &self,
        input: SolanaHistoricalRoots, 
    ) -> Result<(), String> {
        self.historical_roots.validate_era(input)
    }
}

impl EraValidationContext for SolanaHistoricalRoots {
    type EraInput = SolanaHistoricalRoots; 
    type EraOutput = Result<(), String>;

    fn validate_era(&self, input: Self::EraInput) -> Self::EraOutput {

        Ok(())
    }
}
 
