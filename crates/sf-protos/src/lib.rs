//! # StreamingFast's Rust-compiled protocol buffers.
//!
//! This module provides access to Rust implementations of StreamingFast's protocol buffer definitions,
//! enabling the encoding and decoding of data for Ethereum blockchain and bstream services.

/// Module for Ethereum-related data structures and operations.
/// Currently contains the `.proto` defined [here](https://github.com/streamingfast/firehose-ethereum/blob/d9ec696423c2288db640f00026ae29a6cc4c2121/proto/sf/ethereum/type/v2/type.proto#L9)    
pub mod ethereum {
    pub mod r#type {
        pub mod v2 {
            include!(concat!(env!("OUT_DIR"), "/sf.ethereum.r#type.v2.rs"));
        }
    }
}
pub mod bstream {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/sf.bstream.v1.rs"));
    }
}
