
/// A trait defining the context-specific inputs and outputs for era validation.
/// For example, for pre-merge Ethereum we validate eras using the PreMergeAccumulator, while for
/// post-merge Ethereum we validate eras using either HistoricalSummaries or HistoricalRoots. The
/// EraValidationContext trait can be implemented for each of these.

pub trait EraValidationContext {
    type EraInput;
    type EraOutput;

    fn validate_era(&self, input: Self::EraInput) -> Self::EraOutput;
}
