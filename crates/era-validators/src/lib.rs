// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

#![doc = "**DEPRECATED**: This crate has been superseded by the `era-validation` crate.\n\n\
All functionality has been moved to `era-validation`. Please migrate to the new crate.\n\n\
See the [era-validation crate documentation](https://docs.rs/era-validation) for migration guide."]

// ============================================================================
// DEPRECATED: All types and validators below are deprecated in favor of the
// unified `era-validation` crate. This crate now re-exports from
// `era-validation` for backward compatibility.
// ============================================================================

// Deprecated re-exports from era-validation crate

// Generic validator
#[deprecated(
    since = "0.2.0",
    note = "use `era-validation::EraValidatorGeneric` instead"
)]
pub use era_validation::validator::EraValidatorGeneric;

// Ethereum pre-merge
#[deprecated(
    since = "0.2.0",
    note = "use `era-validation::ethereum::EthereumPreMergeValidator` instead"
)]
pub use era_validation::ethereum::EthereumPreMergeValidator;

#[deprecated(
    since = "0.2.0",
    note = "use `era-validation::error::EthereumPreMergeError` instead"
)]
pub use era_validation::error::EthereumPreMergeError as EthereumPreMergeValidatorError;

// Ethereum post-merge
#[deprecated(
    since = "0.2.0",
    note = "use `era-validation::ethereum::EthereumPostMergeValidator` instead"
)]
pub use era_validation::ethereum::EthereumPostMergeValidator;

#[deprecated(
    since = "0.2.0",
    note = "use `era-validation::error::EthereumPostMergeError` instead"
)]
pub use era_validation::error::EthereumPostMergeError;

// Ethereum post-Capella
#[deprecated(
    since = "0.2.0",
    note = "use `era-validation::ethereum::EthereumPostCapellaValidator` instead"
)]
pub use era_validation::ethereum::EthereumPostCapellaValidator;

#[deprecated(
    since = "0.2.0",
    note = "use `era-validation::error::EthereumPostCapellaError` instead"
)]
pub use era_validation::error::EthereumPostCapellaError;

// Solana
#[deprecated(
    since = "0.2.0",
    note = "use `era-validation::solana::SolanaValidator` instead"
)]
pub use era_validation::solana::SolanaValidator;

#[deprecated(
    since = "0.2.0",
    note = "use `era-validation::error::SolanaValidatorError` instead"
)]
pub use era_validation::error::SolanaValidatorError;

// Trait re-export
#[deprecated(
    since = "0.2.0",
    note = "use `era-validation::traits::EraValidationContext` instead"
)]
pub use era_validation::traits::EraValidationContext;
