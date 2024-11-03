use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("Bin code error: {0}")]
    Bincode(#[from] bincode::Error),
    #[error("Incorrect dbin bytes")]
    DbinBytesInvalid,
    #[error("Invalid dbin content type: {0}")]
    DbinContentTypeInvalid(String),
    #[error("Start of new dbin file")]
    DbinMagicBytesInvalid,
    #[error("Unsupported version")]
    DbinVersionUnsupported,
    #[error("Dbin files with different versions")]
    DifferingDbinVersions,
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
    #[error("Failed to match roots for block number {block_number}")]
    MatchRootsFailed { block_number: u64 },
    #[error("Protobuf decode error: {0}")]
    ProtobufDecode(#[from] prost::DecodeError),
    #[error("Invalid Receipt Root")]
    ReceiptRootInvalid,
    #[error("{0}")]
    TokioJoin(#[from] tokio::task::JoinError),
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
}
