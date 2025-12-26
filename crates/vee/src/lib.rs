// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

// 🚀✨ Main Re-exports ✨🚀

#[doc(inline)]
pub use firehose_protos as protos;

#[doc(inline)]
pub use flat_files_decoder as decoder;

#[doc(inline)]
pub use era_validation;

// deprecated: for backward compatibility, re-export header_accumulator
// users should migrate to era-validation crate
#[doc(inline)]
#[allow(deprecated)]
pub use header_accumulator as accumulator;

// convenience re-exports from era-validation
pub use era_validation::ethereum::{
    generate_inclusion_proof, generate_inclusion_proofs, verify_inclusion_proof,
    verify_inclusion_proofs, Epoch, ExtHeaderRecord, HeaderWithProof, InclusionProof,
};
pub use era_validation::EraValidationContext;

// deprecated re-exports for backward compatibility
#[allow(deprecated)]
pub use accumulator::*;

pub use arbitrum_ve::*;
pub use decoder::*;
pub use protos::*;
