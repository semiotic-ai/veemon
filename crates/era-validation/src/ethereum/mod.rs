// SPDX-FileCopyrightText: 2024- Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! ethereum block era validation across all eras

#[cfg(feature = "beacon")]
mod common;
#[cfg(feature = "beacon")]
pub mod post_capella;
#[cfg(feature = "beacon")]
pub mod post_merge;
pub mod pre_merge;
pub mod proof;
pub mod types;

// re-export public types
#[cfg(feature = "beacon")]
pub use post_capella::EthereumPostCapellaValidator;
#[cfg(feature = "beacon")]
pub use post_merge::EthereumPostMergeValidator;
pub use pre_merge::EthereumPreMergeValidator;
pub use proof::{
    generate_inclusion_proof, generate_inclusion_proofs, verify_inclusion_proof,
    verify_inclusion_proofs, HeaderWithProof, InclusionProof,
};
pub use types::{Epoch, ExtHeaderRecord, FINAL_EPOCH, MAX_EPOCH_SIZE, MERGE_BLOCK};

// re-export external types for convenience
pub use ethportal_api::types::execution::accumulator::EpochAccumulator;
pub use validation::HistoricalEpochRoots;
