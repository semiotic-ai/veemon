// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Firehose Ethereum-related data structures and operations.
//! See the protobuffer definitions section of the README for more information.
//!

pub mod access;
pub mod eth_block;
pub mod log;
pub mod transaction;

tonic::include_proto!("sf.ethereum.r#type.v2");
