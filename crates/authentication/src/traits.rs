// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

/// A trait defining the context-specific inputs and outputs for era validation.
///
/// Different blockchain consensus mechanisms and eras require different historical data
/// and validation approaches. This trait provides a generic interface that allows each
/// era to define its own input requirements and output format while maintaining a
/// consistent validation API.
///
/// # Examples
///
/// For pre-merge Ethereum, we validate eras using the PreMergeAccumulator, while for
/// post-merge Ethereum we validate eras using either HistoricalSummaries or HistoricalRoots.
/// Each of these can implement `EraValidationContext` with era-specific types.
///
/// ```rust,ignore
/// impl EraValidationContext for HistoricalEpochRoots {
///     type EraInput = (usize, EpochAccumulator);
///     type EraOutput = Result<(), EthereumPreMergeError>;
///
///     fn validate_era(&self, input: Self::EraInput) -> Self::EraOutput {
///         // era-specific validation logic
///     }
/// }
/// ```
pub trait EraValidationContext {
    /// The input required to validate an era
    ///
    /// This typically includes an era identifier (e.g., epoch number) and the data
    /// to be validated (e.g., block hashes, beacon blocks).
    type EraInput;

    /// The result of era validation
    ///
    /// This is typically a Result type indicating success or failure of validation.
    type EraOutput;

    /// Validates an era against historical trusted data
    ///
    /// # Arguments
    ///
    /// * `input` - Era-specific input containing the data to validate
    ///
    /// # Returns
    ///
    /// The validation result as defined by `EraOutput`
    fn validate_era(&self, input: Self::EraInput) -> Self::EraOutput;
}
