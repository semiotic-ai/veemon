// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Authentication
//!
//! multi-chain block and header authentication library.
//!
//! ## ethereum authentication
//!
//! ethereum has three distinct authentication eras:
//!
//! - **pre-merge (pow)**: blocks 0-15,537,393 - authenticated against HistoricalEpochRoots
//! - **post-merge (pos)**: blocks 15,537,394-17,034,869 - authenticated against HistoricalRoots
//! - **post-capella (pos)**: blocks 17,034,870+ - authenticated against HistoricalSummaries
//!
//! ### quick start
//!
//! ```rust,ignore
//! use authentication::ethereum::{EthereumPreMergeValidator, Epoch};
//! use authentication::traits::EraValidationContext;
//!
//! // create validator with default historical roots
//! let validator = EthereumPreMergeValidator::default();
//!
//! // validate an epoch
//! let epoch: Epoch = headers.try_into()?;
//! let result = validator.validate_era((epoch.number(), epoch.into()))?;
//! ```
//!
//! ## solana authentication
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
pub mod validator;

// re-export core traits
pub use traits::EraValidationContext;

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
    AuthenticationError, EthereumPostCapellaError, EthereumPostMergeError, EthereumPreMergeError,
    SolanaValidatorError,
};

// re-export commonly used validation types
pub use validation::header_validator::HeaderValidator;
pub use validation::historical_roots::HistoricalRootsAccumulator;
pub use validation::PreMergeAccumulator;
