use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtosError {
    #[error("Block conversion error")]
    BlockConversionError,

    #[error("BLS error: {0}")]
    Bls(String),

    #[error("Error in decoding block: {0}")]
    DecodeError(#[from] prost::DecodeError),

    #[error("GraffitiInvalid")]
    GraffitiInvalid,

    #[error("Invalid access tuple storage key: {0}")]
    InvalidAccessTupleStorageKey(String),

    #[error("Invalid BigInt: {0}")]
    InvalidBigInt(String),

    #[error("Invalid log address: {0}")]
    InvalidLogAddress(String),

    #[error("Invalid log topic: {0}")]
    InvalidLogTopic(String),

    #[error("Invalid trace signature {0:?} component: {1}")]
    InvalidTraceSignature(String, String),

    #[error("KzgCommitmentInvalid")]
    KzgCommitmentInvalid,

    #[error("MissingBlockHeader")]
    MissingBlockHeader,

    #[error("Null attestation data")]
    NullAttestationData,

    #[error("Null indexed attestation data")]
    NullIndexedAttestationData,

    #[error("Null block field in block response")]
    NullBlock,

    #[error("Null BlsToExecutionChange")]
    NullBlsToExecutionChange,

    #[error("Null checkpoint")]
    NullCheckpoint,

    #[error("Null deposit data")]
    NullDepositData,

    #[error("Null execution payload")]
    NullExecutionPayload,

    #[error("Proposer Slashing null signer")]
    NullSigner,

    #[error("Null SignedBeaconBlockHeader Message")]
    NullSignedBeaconBlockHeaderMessage,

    #[error("Null voluntary exit")]
    NullVoluntaryExit,

    #[error("SSZ Types error: {0}")]
    SszTypesError(String),

    #[error("Transaction missing call")]
    TransactionMissingCall,

    #[error("TxTypeConversionError: {0}")]
    TxTypeConversion(String),
}
