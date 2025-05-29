// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

/// Get custom error variants for issues with reading, decoding, and verifying flat files.
#[derive(Debug, Error)]
pub enum DecoderError {
    /// [bincode] library error.
    #[error("Bin code error: {0}")]
    Bincode(#[from] bincode::Error),

    /// Flat file bytes invalid.
    #[error("Invalid flat file bytes")]
    BytesInvalid,

    /// Flat file content type invalid.
    #[error("Invalid flat file content type: {0}")]
    ContentTypeInvalid(String),

    /// Error converting from AnyBlock into chain-specific Block.
    #[error("The block contents of this file are not supported")]
    ConversionError,

    /// [firehose_protos] library error.
    #[error("Protos error: {0}")]
    FirehoseProtosError(#[from] firehose_protos::ProtosError),

    /// Format unsupported.
    #[error("Unsupported format: {0:?}")]
    FormatUnsupported(Option<String>),

    /// Header invalid.
    #[error("Invalid header")]
    HeaderInvalid,

    /// [std::io] library error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// [serde_json] library error.
    #[error("{0}")]
    Json(#[from] serde_json::Error),

    /// Magic bytes invalid.
    #[error("Magic bytes at start of file are invalid")]
    MagicBytesInvalid,

    /// Failed to match roots for block number.
    #[error("Failed to match roots for block number {block_number}")]
    MatchRootsFailed {
        /// Block number.
        block_number: u64,
    },

    /// [prost] library decode error.
    #[error("Protobuf decode error: {0}")]
    ProtobufDecode(#[from] prost::DecodeError),

    /// Receipt root invalid.
    #[error("Invalid Receipt Root")]
    ReceiptRootInvalid,

    /// Invalid block header total difficulty.
    #[error("Invalid block header total difficulty")]
    TotalDifficultyInvalid,

    /// Transaction root invalid.
    #[error("Invalid Transaction Root")]
    TransactionRootInvalid,

    /// [std::array::TryFromSliceError].
    #[error("TryFromSliceError: {0}")]
    TryFromSlice(#[from] std::array::TryFromSliceError),

    /// [std::string::FromUtf8Error].
    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Block verification failed for given block.
    #[error("Block verification failed {block_number}")]
    VerificationFailed {
        /// Block number.
        block_number: u64,
    },

    /// Flat files with different versions.
    #[error("Flat files with different versions")]
    VersionConflict,

    /// Unsupported flat file version.
    #[error("Unsupported flat file version")]
    VersionUnsupported,
}
