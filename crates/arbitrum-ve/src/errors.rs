
/// Arbitrum verification errors
#[derive(thiserror::Error, Debug)]
pub enum ArbitrumValidateError {
    /// Error verifying OffchainInclusionProof
    #[error("Error verifying OffchainInclusionProof")]
    OffchainInclusionProofVerificationFailure,
}
