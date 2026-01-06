// SPDX-FileCopyrightText: 2024- Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Era Validation
//!
//! multi-chain block and header era validation library.
//!
//! ## ethereum era validation
//!
//! ethereum has three distinct validation eras:
//!
//! - **pre-merge (pow)**: blocks 0-15,537,393 - validated against HistoricalEpochRoots
//! - **post-merge (pos)**: blocks 15,537,394-17,034,869 - validated against HistoricalRoots
//! - **post-capella (pos)**: blocks 17,034,870+ - validated against HistoricalSummaries
//!
//! ### quick start
//!
//! ```rust,ignore
//! use era_validation::ethereum::{EthereumPreMergeValidator, Epoch};
//! use era_validation::traits::EraValidationContext;
//!
//! // create validator with default historical roots
//! let validator = EthereumPreMergeValidator::default();
//!
//! // validate an epoch
//! let epoch: Epoch = headers.try_into()?;
//! let result = validator.validate_era((epoch.number(), epoch.into()))?;
//! ```
//!
//! ## solana era validation
//!
//! solana eras are defined as 432,000 slot epochs.
//!
//! ## architecture
//!
//! this crate uses trait-based validation with the `EraValidationContext` trait,
//! allowing different blockchain eras and chains to implement their own validation
//! logic while maintaining a consistent interface.

pub mod error;
pub mod ethereum;
pub mod solana;
pub mod traits;
pub mod types;
pub mod validator;

// re-export core traits
pub use traits::EraValidationContext;

// re-export numeric types
pub use types::{BlockNumber, EpochNumber, EraNumber, SlotNumber};

// re-export ethereum types and validators
pub use ethereum::{
    generate_inclusion_proof, generate_inclusion_proofs, verify_inclusion_proof,
    verify_inclusion_proofs, Epoch, EthereumPostCapellaValidator, EthereumPostMergeValidator,
    EthereumPreMergeValidator, ExtHeaderRecord, HeaderWithProof, InclusionProof,
};

// re-export solana types and validators
pub use solana::SolanaValidator;

// re-export generic validator
pub use validator::EraValidatorGeneric;

// re-export errors
pub use error::{
    EraValidationError, EthereumPostCapellaError, EthereumPostMergeError, EthereumPreMergeError,
    SolanaValidatorError,
};

// re-export commonly used validation types
pub use validation::header_validator::HeaderValidator;
pub use validation::historical_roots::HistoricalRootsAccumulator;
pub use validation::PreMergeAccumulator;
