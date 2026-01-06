// SPDX-FileCopyrightText: 2024- Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

#![deny(missing_docs)]
#![doc = "**DEPRECATED**: This crate has been superseded by the `era-validation` crate.\n\n\
All functionality has been moved to `era-validation::ethereum`. Please migrate to the new crate.\n\n\
See the [era-validation crate documentation](https://docs.rs/era-validation) for migration guide."]

// ============================================================================
// DEPRECATED: All types and functions below are deprecated in favor of the
// unified `era-validation` crate. This crate now re-exports from
// `era-validation` for backward compatibility.
// ============================================================================

// Deprecated re-exports from era-validation crate
#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::ethereum::Epoch` instead"
)]
pub use era_validation::ethereum::Epoch;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::ethereum::ExtHeaderRecord` instead"
)]
pub use era_validation::ethereum::ExtHeaderRecord;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::ethereum::MAX_EPOCH_SIZE` instead"
)]
pub use era_validation::ethereum::MAX_EPOCH_SIZE;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::ethereum::FINAL_EPOCH` instead"
)]
pub use era_validation::ethereum::FINAL_EPOCH;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::ethereum::MERGE_BLOCK` instead"
)]
pub use era_validation::ethereum::MERGE_BLOCK;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::ethereum::generate_inclusion_proof` instead"
)]
pub use era_validation::ethereum::generate_inclusion_proof;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::ethereum::generate_inclusion_proofs` instead"
)]
pub use era_validation::ethereum::generate_inclusion_proofs;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::ethereum::verify_inclusion_proof` instead"
)]
pub use era_validation::ethereum::verify_inclusion_proof;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::ethereum::verify_inclusion_proofs` instead"
)]
pub use era_validation::ethereum::verify_inclusion_proofs;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::ethereum::HeaderWithProof` instead"
)]
pub use era_validation::ethereum::HeaderWithProof;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::ethereum::InclusionProof` instead"
)]
pub use era_validation::ethereum::InclusionProof;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::AuthenticationError` instead"
)]
pub use era_validation::error::EraValidationError as EraValidateError;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::error::EthereumPreMergeError` instead"
)]
pub use era_validation::error::EthereumPreMergeError;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::ethereum::EthereumPreMergeValidator` instead"
)]
pub use era_validation::ethereum::EthereumPreMergeValidator as EraValidator;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::ethereum::EpochAccumulator` instead"
)]
pub use era_validation::ethereum::EpochAccumulator;

#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::ethereum::HistoricalEpochRoots` instead"
)]
pub use era_validation::ethereum::HistoricalEpochRoots;

// Re-export PreMergeAccumulator from era-validation crate for convenience
#[deprecated(
    since = "0.4.0",
    note = "use `era-validation::PreMergeAccumulator` directly instead"
)]
pub use era_validation::PreMergeAccumulator;
