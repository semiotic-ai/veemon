// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod dbin;
mod decoder;
mod error;

pub use dbin::*;
pub use decoder::*;
pub use error::*;
