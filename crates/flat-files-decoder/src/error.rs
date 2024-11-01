use thiserror::Error;

/// Get custom error variants for issues with reading, decoding, and verifying flat files.
#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("Bin code error: {0}")]
    Bincode(#[from] bincode::Error),
    #[error("Invalid flat file bytes")]
    BytesInvalid,
    #[error("Invalid flat file content type: {0}")]
    ContentTypeInvalid(String),
    #[error("Protos error: {0}")]
    FirehoseProtosError(#[from] firehose_protos::error::ProtosError),
    #[error("Unsupported format: {0:?}")]
    FormatUnsupported(Option<String>),
    #[error("Invalid header")]
    HeaderInvalid,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("Magic bytes at start of file are invalid")]
    MagicBytesInvalid,
    #[error("Failed to match roots for block number {block_number}")]
    MatchRootsFailed { block_number: u64 },
    #[error("Protobuf decode error: {0}")]
    ProtobufDecode(#[from] prost::DecodeError),
    #[error("Invalid Receipt Root")]
    ReceiptRootInvalid,
    #[error("Invalid block header total difficulty")]
    TotalDifficultyInvalid,
    #[error("Invalid Transaction Root")]
    TransactionRootInvalid,
    #[error("TryFromSliceError: {0}")]
    TryFromSlice(#[from] std::array::TryFromSliceError),
    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Block verification failed {block_number}")]
    VerificationFailed { block_number: u64 },
    #[error("Flat files with different versions")]
    VersionConflict,
    #[error("Unsupported flat file version")]
    VersionUnsupported,
}
