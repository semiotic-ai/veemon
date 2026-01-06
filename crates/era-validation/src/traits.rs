// SPDX-FileCopyrightText: 2024- Semiotic AI, Inc.
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
///     type EraInput = (EpochNumber, EpochAccumulator);
///     type Error = EthereumPreMergeError;
///
///     fn validate_era(&self, input: Self::EraInput) -> Result<(), Self::Error> {
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

    /// The error type returned when validation fails
    ///
    /// Must implement `std::error::Error` to ensure proper error handling.
    type Error: std::error::Error;

    /// Validates an era against historical trusted data
    ///
    /// # Arguments
    ///
    /// * `input` - Era-specific input containing the data to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Validation succeeded
    /// * `Err(Self::Error)` - Validation failed with era-specific error
    fn validate_era(&self, input: Self::EraInput) -> Result<(), Self::Error>;
}
