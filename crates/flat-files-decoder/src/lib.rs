//! # Flat Files Decoder
//!
//! Read, decode, and verify blockchain block flat files.

mod dbin;
mod decoder;
mod error;

pub use dbin::*;
pub use decoder::*;
pub use error::*;
