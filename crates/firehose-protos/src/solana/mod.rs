// SPDX-FileCopyrightText: 2025- Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Firehose Solana-related data structures and operations.
//! See the protobuffer definitions section of the README for more information.
//!

pub mod sol_block;

tonic::include_proto!("sf.solana.r#type.v1");
