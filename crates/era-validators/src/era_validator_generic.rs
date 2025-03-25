

use crate::traits::EraValidationContext;

/// A generic era validator that accepts any type implementing EraValidationContext.
/// By default, it uses HistoricalEpochRoots.
pub struct EraValidatorGeneric<T: EraValidationContext> {
    pub historical_data: T,
}

impl<T: EraValidationContext> EraValidatorGeneric<T> {
    /// Creates a new generic era validator.
    pub fn new(historical_data: T) -> Self {
        Self { historical_data }
    }

    /// Delegates validation to the context-specific implementation.
    pub fn validate_era(&self, input: T::EraInput) -> T::EraOutput {
        self.historical_data.validate_era(input)
    }
}
