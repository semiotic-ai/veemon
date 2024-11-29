// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Header accumulator
//!
//! This crate is used to accumulate block headers and compare them
//! against header accumulators. This process is used to verify the
//! authenticity of these blocks.

#![deny(missing_docs)]

mod epoch;
mod era_validator;
mod errors;
mod inclusion_proof;
mod types;

pub use epoch::*;
pub use era_validator::*;
pub use errors::*;
pub use inclusion_proof::*;
pub use types::*;
