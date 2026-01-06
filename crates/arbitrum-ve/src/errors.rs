// SPDX-FileCopyrightText: 2024- Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Arbitrum verification errors
#[derive(thiserror::Error, Debug)]
pub enum ArbitrumValidateError {
    /// Error verifying OffchainInclusionProof
    #[error("Error verifying OffchainInclusionProof")]
    OffchainInclusionProofVerificationFailure,
}
