//! Type-safe wrappers for blockchain numeric identifiers.
//!
//! This module provides newtype wrappers to prevent accidentally mixing:
//! - Block numbers (execution layer blocks)
//! - Slot numbers (beacon chain slots)
//! - Epoch numbers (8192 pre-merge blocks)
//! - Era numbers (8192 post-merge slots)
//!
//! # Examples
//! ```
//! use era_validation::{BlockNumber, EpochNumber};
//!
//! let block = BlockNumber(16384);
//! let epoch: EpochNumber = block.into();
//! assert_eq!(epoch, EpochNumber(2));
//! ```

use std::fmt;

/// block number in the execution layer (pre and post merge)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockNumber(pub u64);

/// slot number in the beacon chain (post-merge only)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SlotNumber(pub u64);

/// epoch number - represents 8192 blocks in pre-merge ethereum
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EpochNumber(pub u64);

/// era number - represents 8192 slots in post-merge ethereum
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EraNumber(pub u64);

// Display implementations
impl fmt::Display for BlockNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for SlotNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for EpochNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for EraNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// From/Into conversions for BlockNumber
impl From<BlockNumber> for u64 {
    fn from(n: BlockNumber) -> u64 {
        n.0
    }
}

impl From<u64> for BlockNumber {
    fn from(n: u64) -> BlockNumber {
        BlockNumber(n)
    }
}

// From/Into conversions for SlotNumber
impl From<SlotNumber> for u64 {
    fn from(n: SlotNumber) -> u64 {
        n.0
    }
}

impl From<u64> for SlotNumber {
    fn from(n: u64) -> SlotNumber {
        SlotNumber(n)
    }
}

// From/Into conversions for EpochNumber
impl From<EpochNumber> for u64 {
    fn from(n: EpochNumber) -> u64 {
        n.0
    }
}

impl From<u64> for EpochNumber {
    fn from(n: u64) -> EpochNumber {
        EpochNumber(n)
    }
}

impl From<usize> for EpochNumber {
    fn from(n: usize) -> EpochNumber {
        EpochNumber(n as u64)
    }
}

impl From<EpochNumber> for usize {
    fn from(n: EpochNumber) -> usize {
        n.0 as usize
    }
}

// From/Into conversions for EraNumber
impl From<EraNumber> for u64 {
    fn from(n: EraNumber) -> u64 {
        n.0
    }
}

impl From<u64> for EraNumber {
    fn from(n: u64) -> EraNumber {
        EraNumber(n)
    }
}

impl From<usize> for EraNumber {
    fn from(n: usize) -> EraNumber {
        EraNumber(n as u64)
    }
}

impl From<EraNumber> for usize {
    fn from(n: EraNumber) -> usize {
        n.0 as usize
    }
}

// Block/Slot to Epoch/Era conversions
impl From<BlockNumber> for EpochNumber {
    fn from(block: BlockNumber) -> EpochNumber {
        EpochNumber(block.0 / 8192)
    }
}

impl From<SlotNumber> for EraNumber {
    fn from(slot: SlotNumber) -> EraNumber {
        EraNumber(slot.0 / 8192)
    }
}

// Division by constants for conversions
impl std::ops::Div<u64> for BlockNumber {
    type Output = EpochNumber;
    fn div(self, rhs: u64) -> EpochNumber {
        EpochNumber(self.0 / rhs)
    }
}

impl std::ops::Div<u64> for SlotNumber {
    type Output = EraNumber;
    fn div(self, rhs: u64) -> EraNumber {
        EraNumber(self.0 / rhs)
    }
}

// Remainder for alignment checking
impl std::ops::Rem<u64> for SlotNumber {
    type Output = u64;
    fn rem(self, rhs: u64) -> u64 {
        self.0 % rhs
    }
}

// Subtraction for adjacent block checks
impl std::ops::Sub for BlockNumber {
    type Output = u64;
    fn sub(self, rhs: BlockNumber) -> u64 {
        self.0 - rhs.0
    }
}

// Subtraction for era index adjustments (Capella fork)
impl std::ops::Sub<usize> for EraNumber {
    type Output = usize;
    fn sub(self, rhs: usize) -> usize {
        self.0 as usize - rhs
    }
}
