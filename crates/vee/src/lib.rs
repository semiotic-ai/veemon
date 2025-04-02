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
pub use header_accumulator as accumulator;

pub use accumulator::*;
pub use decoder::*;
pub use protos::*;

mod proof;
