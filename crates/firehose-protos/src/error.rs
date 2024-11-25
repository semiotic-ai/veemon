// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

/// Custom error variants for Verifiable Extraction protobuffer types.
#[derive(Error, Debug)]
pub enum ProtosError {
    /// Invalid access tuple storage key.
    #[error("Invalid access tuple storage key: {0}")]
    AccessTupleStorageKeyInvalid(String),

    /// Missing attestation data.
    #[error("Null attestation data")]
    AttestationDataMissing,

    /// Invalid BigInt.
    #[error("Invalid BigInt: {0}")]
    BigIntInvalid(String),

    /// Error converting protobuffer to block type.
    #[error("Block conversion error")]
    BlockConversionError,

    /// Converted block missing block header.
    #[error("BlockHeaderMissing")]
    BlockHeaderMissing,

    /// Block response missing block.
    #[error("Null block field in block response")]
    BlockMissingInResponse,

    /// [BLS signature](https://en.wikipedia.org/wiki/BLS_digital_signature) error.
    #[error("BLS error: {0}")]
    Bls(String),

    /// Missing BLS to Execution Change
    #[error("Null BlsToExecutionChange")]
    BlsToExecutionChangeMissing,

    /// Checkpoint missing.
    #[error("Null checkpoint")]
    CheckpointMissing,

    /// [prost] library decode error.
    #[error("Error in decoding block: {0}")]
    DecodeError(#[from] prost::DecodeError),

    /// Missing deposit data.
    #[error("Null deposit data")]
    DepositDataMissing,

    /// Missing execution payload.
    #[error("Null execution payload")]
    ExecutionPayloadMissing,

    /// Graffiti invalid when decoding block.
    #[error("GraffitiInvalid")]
    GraffitiInvalid,

    /// Missing indexed attestation data.
    #[error("Null indexed attestation data")]
    IndexedAttestationDataMissing,

    /// Invalid KZG commitment.
    #[error("KzgCommitmentInvalid")]
    KzgCommitmentInvalid,

    /// Invalid log address.
    #[error("Invalid log address: {0}")]
    LogAddressInvalid(String),

    /// Invalid log topic.
    #[error("Invalid log topic: {0}")]
    LogTopicInvalid(String),

    /// Missing signed Beacon block header message.
    #[error("Null SignedBeaconBlockHeader Message")]
    SignedBeaconBlockHeaderMessageMissing,

    /// Missing signer
    #[error("Null signer")]
    SignerMissing,

    /// Invalid trace signature for ECDSA component.
    #[error("Invalid trace signature {0:?} component: {1}")]
    TraceSignatureInvalid(String, String),

    /// SSZ Types error.
    #[error("SSZ Types error: {0}")]
    SszTypesError(String),

    /// Transaction missing call.
    #[error("Transaction missing call")]
    TransactionMissingCall,

    /// Invalid transaction receipt logs bloom.
    #[error("Invalid transaction receipt logs bloom: {0}")]
    TransactionReceiptLogsBloomInvalid(String),

    /// Transaction trace missing receipt.
    #[error("Transaction trace missing receipt")]
    TransactionTraceMissingReceipt,

    /// Transaction type conversion error.
    #[error("TxTypeConversionError: {0}")]
    TxTypeConversion(String),

    /// Missing voluntary exit.
    #[error("Null voluntary exit")]
    VoluntaryExitMissing,
}
