use thiserror::Error;

use crate::ethereum_v2::EcdsaComponent;

#[derive(Error, Debug)]
pub enum ProtosError {
    #[error("Block conversion error")]
    BlockConversionError,

    #[error("Invalid address: {0}")]
    BlockLogInvalidAddressError(String),

    #[error("Invalid topic: {0}")]
    BlockLogInvalidTopicError(String),

    #[error("TryFromSliceError: {0}")]
    BlockLogTryFromSliceError(#[from] std::array::TryFromSliceError),

    #[error("BLS error: {0}")]
    Bls(String),

    #[error("Error in decoding block: {0}")]
    DecodeError(#[from] prost::DecodeError),

    #[error("GraffitiInvalid")]
    GraffitiInvalid,

    #[error("Invalid BigInt: {0}")]
    InvalidBigInt(String),

    #[error("Invalid Storage Key: {0}")]
    InvalidStorageKey(String),

    #[error("Invalid trace signature {0:?} component: {1}")]
    InvalidTraceSignature(EcdsaComponent, String),

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

    #[error("Transaction call missing")]
    TransactionMissingCall,

    #[error("Transaction type conversion error")]
    TransactionTypeConversionError,
}
