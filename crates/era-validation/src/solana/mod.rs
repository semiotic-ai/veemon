// SPDX-FileCopyrightText: 2024- Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! solana block era validation

pub mod validator;

// re-export public types
pub use validator::{SolanaHistoricalRoots, SolanaValidator};
