// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Flat Files Decoder
//!
//! Read, decode, and verify blockchain block flat files.

#![deny(missing_docs)]

mod dbin;
mod decoder;
mod error;

pub use dbin::*;
pub use decoder::*;
pub use error::*;
