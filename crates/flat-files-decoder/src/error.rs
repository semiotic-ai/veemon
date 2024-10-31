use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("Bin code error: {0}")]
    Bincode(#[from] bincode::Error),
    #[error("dbin files with different versions")]
    DifferingDbinVersions,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid content type: {0}")]
    InvalidContentType(String),
    #[error("Incorrect dbin bytes")]
    InvalidDbinBytes,
    #[error("Invalid header")]
    InvalidHeader,
    #[error("Invalid input")]
    InvalidInput,
    #[error("Invalid block header total difficulty")]
    InvalidTotalDifficulty,
    #[error("Invalid UTF8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Block header JSON Error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Mismatched roots for block number {block_number}")]
    MismatchedRoots { block_number: u64 },
    #[error("Protobuf decode error: {0}")]
    ProtobufDecode(#[from] prost::DecodeError),
    #[error("Protos error: {0}")]
    ProtosError(#[from] firehose_protos::error::ProtosError),
    #[error("Invalid Receipt Root")]
    ReceiptRoot,
    #[error("Start of new dbin file")]
    StartOfNewDbinFile,
    #[error("Invalid Transaction Root")]
    TransactionRoot,
    #[error("TryFromSliceError: {0}")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    #[error("Unsupported version")]
    UnsupportedDbinVersion,
}
