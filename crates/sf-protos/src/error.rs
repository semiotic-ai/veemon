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

    #[error("KzgCommitmentInvalid")]
    KzgCommitmentInvalid,

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
}
