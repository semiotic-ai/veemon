// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Generic era validator wrapper
//!
//! This module provides a generic wrapper for any type implementing the
//! `EraValidationContext` trait.

use crate::traits::EraValidationContext;

/// A generic era validator that wraps any type implementing `EraValidationContext`.
///
/// This allows for polymorphic validation across different blockchain eras and chains
/// while maintaining type safety.
///
/// # Type Parameters
///
/// * `T` - Any type implementing `EraValidationContext`
///
/// # Examples
///
/// ```rust,ignore
/// use era_validation::{EraValidatorGeneric, ethereum::EthereumPreMergeValidator};
/// use validation::HistoricalEpochRoots;
///
/// let historical_roots = HistoricalEpochRoots::default();
/// let validator = EraValidatorGeneric::new(historical_roots);
/// ```
#[derive(Debug)]
pub struct EraValidatorGeneric<T: EraValidationContext> {
    context: T,
}

impl<T: EraValidationContext> EraValidatorGeneric<T> {
    /// Creates a new generic era validator with the given context
    pub fn new(context: T) -> Self {
        Self { context }
    }

    /// Validates an era using the wrapped context
    pub fn validate_era(&self, input: T::EraInput) -> T::EraOutput {
        self.context.validate_era(input)
    }
}
