// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Verifiable Extraction Protocol Buffers in Rust
//!
//! This module provides Rust implementations of StreamingFast's protocol buffer definitions,
//! enabling efficient encoding and decoding of data for Ethereum blockchain services and Firehose,
//! StreamingFast’s framework for streaming blockchain data.
//!
//! ## Usage
//!
//! Check out [`veemon/firehose-client`](../firehose_client/index.html) for a high-level client
//! that you can use with chain data endpoint providers like Pinax or StremaingFast.
//!
//! Alternatively, for tools for ingesting these block types from flat files, check out
//! [`veemon/flat-files-decoder`](../flat-files-decoder/index.html).

#![deny(missing_docs)]

mod error;
mod ethereum_v2;

mod bstream {
    pub mod v1 {
        tonic::include_proto!("sf.bstream.v1");
    }
}

pub use bstream::v1::Block as BstreamBlock;
pub use error::ProtosError;
pub use ethereum_v2::{eth_block::FullReceipt, Block as EthBlock, BlockHeader};
