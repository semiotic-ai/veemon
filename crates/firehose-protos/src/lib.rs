// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

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
