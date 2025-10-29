// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Type-safe wrappers for post-Capella verification.
//!
//! Zero-cost newtypes encoding domain concepts in the type system.
//! All sizes verified at compile-time via static assertions.

use anyhow::anyhow;

use crate::constants::{CAPELLA_FORK_EPOCH, EPOCH_SIZE, SLOTS_PER_EPOCH};

/// Beacon chain slot number (12 second intervals).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct BeaconSlot(u64);

impl BeaconSlot {
    #[inline(always)]
    pub const fn new(slot: u64) -> Self {
        Self(slot)
    }

    #[inline(always)]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Position within epoch (0-8191): `slot % EPOCH_SIZE`
    #[inline]
    pub fn block_root_index(&self) -> BlockRootIndex {
        BlockRootIndex(self.0 % EPOCH_SIZE)
    }

    /// Converts to historical summary index, validating slot >= Capella fork and within bounds.
    ///
    /// Formula: `(slot - capella_start_slot) / EPOCH_SIZE`
    pub fn to_historical_summary_index(
        &self,
        summaries_len: usize,
    ) -> anyhow::Result<ValidatedHistoricalSummaryIndex> {
        let capella_start_slot = CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH;

        if self.0 < capella_start_slot {
            return Err(anyhow!(
                "slot {} is before capella fork epoch (slot {})",
                self.0,
                capella_start_slot
            ));
        }

        let relative_slot = self.0 - capella_start_slot;
        let index = (relative_slot / EPOCH_SIZE) as usize;

        if index >= summaries_len {
            return Err(anyhow!(
                "historical summary index {} out of bounds (max {})",
                index,
                summaries_len - 1
            ));
        }

        Ok(ValidatedHistoricalSummaryIndex { index })
    }
}

/// Block root position within epoch (0-8191).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct BlockRootIndex(u64);

impl BlockRootIndex {
    #[inline(always)]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// SSZ Merkle tree generalized index: `EPOCH_SIZE + block_root_index`
    #[inline]
    pub fn generalized_index(&self) -> GeneralizedIndex {
        GeneralizedIndex(EPOCH_SIZE + self.0)
    }
}

/// SSZ Merkle tree path index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct GeneralizedIndex(u64);

impl GeneralizedIndex {
    #[inline(always)]
    pub const fn new(index: u64) -> Self {
        Self(index)
    }

    #[inline(always)]
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

/// Bounds-checked historical summaries index (parse, don't validate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ValidatedHistoricalSummaryIndex {
    index: usize,
}

impl ValidatedHistoricalSummaryIndex {
    #[inline(always)]
    pub const fn as_usize(&self) -> usize {
        self.index
    }
}

/// Merkle proof depth for beacon block roots (8192 = 2^13).
pub const BEACON_BLOCK_PROOF_DEPTH: usize = 13;

/// Path to execution block hash: BeaconBlock → body → execution_payload → block_hash
pub const EXECUTION_BLOCK_GENERALIZED_INDEX: GeneralizedIndex = GeneralizedIndex::new(3228);

// Static assertions to prove zero-cost abstractions at compile time
static_assertions::assert_eq_size!(BeaconSlot, u64);
static_assertions::assert_eq_size!(BlockRootIndex, u64);
static_assertions::assert_eq_size!(GeneralizedIndex, u64);
static_assertions::assert_eq_size!(ValidatedHistoricalSummaryIndex, usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beacon_slot_block_root_index() {
        // slot exactly at epoch boundary
        let slot = BeaconSlot::new(CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH);
        assert_eq!(slot.block_root_index().as_u64(), 0);

        // slot in middle of epoch
        let slot = BeaconSlot::new(CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH + 4096);
        assert_eq!(slot.block_root_index().as_u64(), 4096);

        // slot at end of epoch
        let slot = BeaconSlot::new(CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH + 8191);
        assert_eq!(slot.block_root_index().as_u64(), 8191);
    }

    #[test]
    fn block_root_index_generalized_index() {
        let slot = BeaconSlot::new(CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH);
        let block_root_index = slot.block_root_index();
        let gen_index = block_root_index.generalized_index();

        // first block in epoch: EPOCH_SIZE + 0
        assert_eq!(gen_index.as_usize(), EPOCH_SIZE as usize);

        // last block in epoch: EPOCH_SIZE + 8191
        let slot = BeaconSlot::new(CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH + 8191);
        let gen_index = slot.block_root_index().generalized_index();
        assert_eq!(gen_index.as_usize(), (EPOCH_SIZE + 8191) as usize);
    }

    #[test]
    fn historical_summary_index_pre_capella_fails() {
        // slot before capella should fail
        let slot = BeaconSlot::new(CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH - 1);
        let result = slot.to_historical_summary_index(100);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("before capella"));
    }

    #[test]
    fn historical_summary_index_out_of_bounds_fails() {
        // slot that would result in index >= summaries_len
        let slot = BeaconSlot::new(CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH + 10 * EPOCH_SIZE);
        let result = slot.to_historical_summary_index(5); // only 5 summaries available

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn historical_summary_index_valid() {
        // first slot after capella
        let slot = BeaconSlot::new(CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH);
        let result = slot.to_historical_summary_index(10);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_usize(), 0);

        // slot in second summary period
        let slot = BeaconSlot::new(CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH + EPOCH_SIZE);
        let result = slot.to_historical_summary_index(10);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_usize(), 1);
    }

    #[test]
    fn execution_block_generalized_index_constant() {
        // verify the constant has the expected value
        assert_eq!(EXECUTION_BLOCK_GENERALIZED_INDEX.as_usize(), 3228);
    }
}
