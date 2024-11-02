//! # Flat Files Decoder for Firehose
//!
//! This crate offers utility functions for reading and verifying flat files stored on disk.
//! The verifier checks computed receipts and transaction roots against those specified in the
//! block header. Additionally, it can optionally verify the block headers against a directory
//! of JSON-formatted block headers.

pub mod compression;
pub mod dbin;
pub mod decoder;
pub mod error;
