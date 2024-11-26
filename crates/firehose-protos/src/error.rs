use thiserror::Error;

/// Custom error variants for Verifiable Extraction protobuffer types.
#[derive(Error, Debug)]
pub enum ProtosError {
    /// Error converting protobuffer to block type.
    #[error("Block conversion error")]
    BlockConversionError,

    /// [BLS signature](https://en.wikipedia.org/wiki/BLS_digital_signature) error.
    #[error("BLS error: {0}")]
    Bls(String),

    /// [prost] library decode error.
    #[error("Error in decoding block: {0}")]
    DecodeError(#[from] prost::DecodeError),

    /// Graffiti invalid when decoding block.
    #[error("GraffitiInvalid")]
    GraffitiInvalid,

    /// Invalid access tuple storage key.
    #[error("Invalid access tuple storage key: {0}")]
    InvalidAccessTupleStorageKey(String),

    /// Invalid BigInt.
    #[error("Invalid BigInt: {0}")]
    InvalidBigInt(String),

    /// Invalid log address.
    #[error("Invalid log address: {0}")]
    InvalidLogAddress(String),

    /// Invalid log topic.
    #[error("Invalid log topic: {0}")]
    InvalidLogTopic(String),

    /// Invalid trace signature for ECDSA component.
    #[error("Invalid trace signature {0:?} component: {1}")]
    InvalidTraceSignature(String, String),

    /// Invalid transaction receipt logs bloom.
    #[error("Invalid transaction receipt logs bloom: {0}")]
    InvalidTransactionReceiptLogsBloom(String),

    /// Invalid KZG commitment.
    #[error("KzgCommitmentInvalid")]
    KzgCommitmentInvalid,

    /// Converted block missing block header.
    #[error("MissingBlockHeader")]
    MissingBlockHeader,

    /// Missing attestation data.
    #[error("Null attestation data")]
    NullAttestationData,

    /// Missing indexed attestation data.
    #[error("Null indexed attestation data")]
    NullIndexedAttestationData,

    /// Block response missing block.
    #[error("Null block field in block response")]
    NullBlock,

    /// Missing BLS to Execution Change
    #[error("Null BlsToExecutionChange")]
    NullBlsToExecutionChange,

    /// Checkpoint missing.
    #[error("Null checkpoint")]
    NullCheckpoint,

    /// Missing deposit data.
    #[error("Null deposit data")]
    NullDepositData,

    /// Missing execution payload.
    #[error("Null execution payload")]
    NullExecutionPayload,

    /// Missing signer
    #[error("Null signer")]
    NullSigner,

    /// Missing signed Beacon block header message.
    #[error("Null SignedBeaconBlockHeader Message")]
    NullSignedBeaconBlockHeaderMessage,

    /// Missing voluntary exit.
    #[error("Null voluntary exit")]
    NullVoluntaryExit,

    /// SSZ Types error.
    #[error("SSZ Types error: {0}")]
    SszTypesError(String),

    /// Transaction missing call.
    #[error("Transaction missing call")]
    TransactionMissingCall,

    /// Transaction trace missing receipt.
    #[error("Transaction trace missing receipt")]
    TransactionTraceMissingReceipt,

    /// Transaction type conversion error.
    #[error("TxTypeConversionError: {0}")]
    TxTypeConversion(String),
}
