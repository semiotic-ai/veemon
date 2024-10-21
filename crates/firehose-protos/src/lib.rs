//! # StreamingFast's Rust-compiled protocol buffers.
//!
//! This module provides access to Rust implementations of StreamingFast's protocol buffer definitions,
//! enabling the encoding and decoding of data for Ethereum blockchain and bstream services.

pub mod beacon_v1;
pub mod error;
pub mod ethereum_v2;

pub mod bstream {
    pub mod v1 {
        tonic::include_proto!("sf.bstream.v1");
    }
}

pub mod firehose {
    pub mod v2 {
        tonic::include_proto!("sf.firehose.v2");
    }
}
