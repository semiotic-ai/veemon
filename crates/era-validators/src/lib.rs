// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

#![doc = "**DEPRECATED**: This crate has been superseded by the `authentication` crate.\n\n\
All functionality has been moved to `authentication`. Please migrate to the new crate.\n\n\
See the [authentication crate documentation](https://docs.rs/authentication) for migration guide."]

// ============================================================================
// DEPRECATED: All types and validators below are deprecated in favor of the
// unified `authentication` crate. This crate now re-exports from
// `authentication` for backward compatibility.
// ============================================================================

// Deprecated re-exports from authentication crate

// Generic validator
#[deprecated(
    since = "0.2.0",
    note = "use `authentication::EraValidatorGeneric` instead"
)]
pub use authentication::validator::EraValidatorGeneric;

// Ethereum pre-merge
#[deprecated(
    since = "0.2.0",
    note = "use `authentication::ethereum::EthereumPreMergeValidator` instead"
)]
pub use authentication::ethereum::EthereumPreMergeValidator;

#[deprecated(
    since = "0.2.0",
    note = "use `authentication::error::EthereumPreMergeError` instead"
)]
pub use authentication::error::EthereumPreMergeError as EthereumPreMergeValidatorError;

// Ethereum post-merge
#[deprecated(
    since = "0.2.0",
    note = "use `authentication::ethereum::EthereumPostMergeValidator` instead"
)]
pub use authentication::ethereum::EthereumPostMergeValidator;

#[deprecated(
    since = "0.2.0",
    note = "use `authentication::error::EthereumPostMergeError` instead"
)]
pub use authentication::error::EthereumPostMergeError;

// Ethereum post-Capella
#[deprecated(
    since = "0.2.0",
    note = "use `authentication::ethereum::EthereumPostCapellaValidator` instead"
)]
pub use authentication::ethereum::EthereumPostCapellaValidator;

#[deprecated(
    since = "0.2.0",
    note = "use `authentication::error::EthereumPostCapellaError` instead"
)]
pub use authentication::error::EthereumPostCapellaError;

// Solana
#[deprecated(
    since = "0.2.0",
    note = "use `authentication::solana::SolanaValidator` instead"
)]
pub use authentication::solana::SolanaValidator;

#[deprecated(
    since = "0.2.0",
    note = "use `authentication::error::SolanaValidatorError` instead"
)]
pub use authentication::error::SolanaValidatorError;

// Trait re-export
#[deprecated(
    since = "0.2.0",
    note = "use `authentication::traits::EraValidationContext` instead"
)]
pub use authentication::traits::EraValidationContext;
