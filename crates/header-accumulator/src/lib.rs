// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

#![deny(missing_docs)]
#![doc = "**DEPRECATED**: This crate has been superseded by the `authentication` crate.\n\n\
All functionality has been moved to `authentication::ethereum`. Please migrate to the new crate.\n\n\
See the [authentication crate documentation](https://docs.rs/authentication) for migration guide."]

// ============================================================================
// DEPRECATED: All types and functions below are deprecated in favor of the
// unified `authentication` crate. This crate now re-exports from
// `authentication` for backward compatibility.
// ============================================================================

// Deprecated re-exports from authentication crate
#[deprecated(
    since = "0.4.0",
    note = "use `authentication::ethereum::Epoch` instead"
)]
pub use authentication::ethereum::Epoch;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::ethereum::ExtHeaderRecord` instead"
)]
pub use authentication::ethereum::ExtHeaderRecord;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::ethereum::MAX_EPOCH_SIZE` instead"
)]
pub use authentication::ethereum::MAX_EPOCH_SIZE;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::ethereum::FINAL_EPOCH` instead"
)]
pub use authentication::ethereum::FINAL_EPOCH;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::ethereum::MERGE_BLOCK` instead"
)]
pub use authentication::ethereum::MERGE_BLOCK;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::ethereum::generate_inclusion_proof` instead"
)]
pub use authentication::ethereum::generate_inclusion_proof;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::ethereum::generate_inclusion_proofs` instead"
)]
pub use authentication::ethereum::generate_inclusion_proofs;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::ethereum::verify_inclusion_proof` instead"
)]
pub use authentication::ethereum::verify_inclusion_proof;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::ethereum::verify_inclusion_proofs` instead"
)]
pub use authentication::ethereum::verify_inclusion_proofs;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::ethereum::HeaderWithProof` instead"
)]
pub use authentication::ethereum::HeaderWithProof;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::ethereum::InclusionProof` instead"
)]
pub use authentication::ethereum::InclusionProof;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::AuthenticationError` instead"
)]
pub use authentication::error::AuthenticationError as EraValidateError;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::error::EthereumPreMergeError` instead"
)]
pub use authentication::error::EthereumPreMergeError;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::ethereum::EthereumPreMergeValidator` instead"
)]
pub use authentication::ethereum::EthereumPreMergeValidator as EraValidator;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::ethereum::EpochAccumulator` instead"
)]
pub use authentication::ethereum::EpochAccumulator;

#[deprecated(
    since = "0.4.0",
    note = "use `authentication::ethereum::HistoricalEpochRoots` instead"
)]
pub use authentication::ethereum::HistoricalEpochRoots;

// Re-export PreMergeAccumulator from authentication crate for convenience
#[deprecated(
    since = "0.4.0",
    note = "use `authentication::PreMergeAccumulator` directly instead"
)]
pub use authentication::PreMergeAccumulator;
