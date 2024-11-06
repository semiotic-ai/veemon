use merkle_proof::MerkleTree;
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tree_hash::TreeHash;
use types::{
    historical_summary::HistoricalSummary, light_client_update, map_beacon_state_altair_fields,
    map_beacon_state_base_fields, map_beacon_state_bellatrix_fields,
    map_beacon_state_capella_fields, map_beacon_state_deneb_fields,
    map_beacon_state_electra_fields, BeaconBlockHeader, BeaconState, BeaconStateAltair,
    BeaconStateBase, BeaconStateBellatrix, BeaconStateCapella, BeaconStateDeneb,
    BeaconStateElectra, BeaconStateError as Error, BitVector, Checkpoint, Epoch, Eth1Data, EthSpec,
    ExecutionPayloadHeaderBellatrix, ExecutionPayloadHeaderCapella, ExecutionPayloadHeaderDeneb,
    ExecutionPayloadHeaderElectra, Fork, Hash256, List, ParticipationFlags, PendingAttestation,
    PendingBalanceDeposit, PendingConsolidation, PendingPartialWithdrawal, Slot, SyncCommittee,
    Validator, Vector,
};

/// The number of slots in an epoch.
pub const SLOTS_PER_EPOCH: usize = 32;
/// The number of slots in an era.
pub const SLOTS_PER_ERA: usize = SLOTS_PER_HISTORICAL_ROOT;
/// Slots are 0-indexed.
/// See, for example, `https://beaconcha.in/slot/0`.
pub const BEACON_GENESIS_SLOT: usize = 0;
/// See [Upgrading Ethereum](https://eth2book.info/capella/part4/history/) for more information.
pub const PHASE_0_START_EPOCH: usize = 0;
/// See [Upgrading Ethereum](https://eth2book.info/capella/part4/history/) for more information.
pub const ALTAIR_START_EPOCH: usize = 74240;
/// See [Upgrading Ethereum](https://eth2book.info/capella/part4/history/) for more information.
pub const BELLATRIX_START_EPOCH: usize = 144896;
/// See [Upgrading Ethereum](https://eth2book.info/capella/part4/history/) for more information.
pub const CAPELLA_START_EPOCH: usize = 194048;
/// See [Upgrading Ethereum](https://eth2book.info/capella/part4/history/) for more information.
/// The first slot number of the Deneb fork.
pub const CAPELLA_START_SLOT: usize = CAPELLA_START_EPOCH * SLOTS_PER_EPOCH;
/// The first era of the Deneb fork.
pub const CAPELLA_START_ERA: usize =
    (CAPELLA_START_EPOCH * SLOTS_PER_EPOCH) / SLOTS_PER_HISTORICAL_ROOT;
/// <https://beaconcha.in/slot/8626176>
pub const DENEB_START_SLOT: usize = 8626176;
/// <https://beaconcha.in/slot/8626176>
pub const FIRST_EXECUTION_BLOCK_DENEB: usize = 19426587;
/// The offset between the Ethereum block number and the Beacon block number at the start of the Deneb fork,
/// i.e. the difference between the first execution block number in the Deneb fork and the start slot number of the Deneb fork.
pub const ETHEREUM_BEACON_DENEB_OFFSET: usize = FIRST_EXECUTION_BLOCK_DENEB - DENEB_START_SLOT;

/// [`BeaconState`] `block_roots` vector has length [`SLOTS_PER_HISTORICAL_ROOT`] (See <https://github.com/ethereum/consensus-specs/blob/dev/specs/capella/beacon-chain.md#beaconstate>),
/// the value of which is calculated uint64(2**13) (= 8,192) (See <https://eth2book.info/capella/part3/config/preset/#time-parameters>)
pub const HISTORY_TREE_DEPTH: usize = 13;

/// The historical roots tree (pre-Capella) and the historical summaries tree (post-Capella) have the same depth.
/// Both tree's root has the block_roots tree root and the state_roots tree root as children and so has one more layer than each of these trees.
pub const HISTORICAL_SUMMARY_TREE_DEPTH: usize = 14;

/// Historical roots is a top-level field on [`BeaconState`], subtract off the generalized indices
// for the internal nodes. Result should be 7, the field offset of the committee in the [`BeaconState`]:
// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#beaconstate
pub const HISTORICAL_ROOTS_INDEX: usize = 39;

/// Historical summaries is a top-level field on [`BeaconState`], subtract off the generalized indices
// for the internal nodes. Result should be 27, the field offset of the committee in the [`BeaconState`]:
// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#beaconstate
pub const HISTORICAL_SUMMARIES_INDEX: usize = 59;

/// Index of `historical_roots` field in the [`BeaconState`] [struct](https://github.com/ethereum/annotated-spec/blob/master/phase0/beacon-chain.md#beaconstate).
pub const HISTORICAL_ROOTS_FIELD_INDEX: usize = 7;

/// Index of `historical_summaries` field in the (post-Capella) [`BeaconState`] [struct](https://github.com/ethereum/annotated-spec/blob/master/capella/beacon-chain.md#beaconstate).
pub const HISTORICAL_SUMMARIES_FIELD_INDEX: usize = 27;

/// The maximum number of block roots that can be stored in a [`BeaconState`]'s `block_roots` list.
pub const SLOTS_PER_HISTORICAL_ROOT: usize = 8192;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HeadState<E: EthSpec> {
    version: String,
    execution_optimistic: bool,
    data: BeaconState<E>,
}

impl<E: EthSpec> HeadState<E> {
    pub fn compute_merkle_proof_for_historical_data(
        &self,
        index: usize,
    ) -> Result<Vec<H256>, Error> {
        // 1. Convert generalized index to field index.
        let field_index = match index {
            HISTORICAL_ROOTS_INDEX | HISTORICAL_SUMMARIES_INDEX => index
                .checked_sub(self.data.num_fields_pow2())
                .ok_or(Error::IndexNotSupported(index))?,
            _ => return Err(Error::IndexNotSupported(index)),
        };

        // 2. Get all `BeaconState` leaves.
        let mut leaves = vec![];
        #[allow(clippy::arithmetic_side_effects)]
        match &self.data {
            BeaconState::Base(state) => {
                map_beacon_state_base_fields!(state, |_, field| {
                    leaves.push(field.tree_hash_root());
                });
            }
            BeaconState::Altair(state) => {
                map_beacon_state_altair_fields!(state, |_, field| {
                    leaves.push(field.tree_hash_root());
                });
            }
            BeaconState::Bellatrix(state) => {
                map_beacon_state_bellatrix_fields!(state, |_, field| {
                    leaves.push(field.tree_hash_root());
                });
            }
            BeaconState::Capella(state) => {
                map_beacon_state_capella_fields!(state, |_, field| {
                    leaves.push(field.tree_hash_root());
                });
            }
            BeaconState::Deneb(state) => {
                map_beacon_state_deneb_fields!(state, |_, field| {
                    leaves.push(field.tree_hash_root());
                });
            }
            BeaconState::Electra(state) => {
                map_beacon_state_electra_fields!(state, |_, field| {
                    leaves.push(field.tree_hash_root());
                });
            }
        };

        // 3. Make deposit tree.
        // Use the depth of the `BeaconState` fields (i.e. `log2(32) = 5`).
        let depth = light_client_update::CURRENT_SYNC_COMMITTEE_PROOF_LEN;
        let tree = MerkleTree::create(&leaves, depth);
        let (_, proof) = tree.generate_proof(field_index, depth)?;

        Ok(proof)
    }

    pub fn data(&self) -> &BeaconState<E> {
        &self.data
    }

    pub fn execution_optimistic(&self) -> bool {
        self.execution_optimistic
    }

    pub fn historical_roots_tree_hash_root(&self) -> H256 {
        self.data.historical_roots().tree_hash_root()
    }

    pub fn historical_summaries_tree_hash_root(&self) -> Result<H256, Error> {
        Ok(self.data.historical_summaries()?.tree_hash_root())
    }

    pub fn state_root(&mut self) -> Result<Hash256, Error> {
        self.data.canonical_root()
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    /// This computation only makes sense if we have all of the leaves (BeaconBlock roots) to construct
    /// the [`HistoricalSummary`] Merkle tree.
    /// We construct a new [`HistoricalSummary`] from the state and check that the tree root is in historical_summaries.
    /// This will be true if the state is in the first slot of an era.
    pub fn block_roots_contain_entire_era(&self) -> Result<bool, Error> {
        // Check if the block_roots buffer can have accumulated an entire era, i.e. 8192 blocks.
        if self.data.block_roots().len() % SLOTS_PER_HISTORICAL_ROOT == 0 {
            let historical_summary = HistoricalSummary::new(&self.data);
            Ok(self
                .data
                .historical_summaries()?
                .iter()
                .last()
                .map(|summary| summary == &historical_summary)
                .unwrap_or(false))
        } else {
            Ok(false)
        }
    }

    /// Computes a Merkle inclusion proof of a `BeaconBlock` root using Merkle trees from either
    /// the [`historical_roots`](https://github.com/ethereum/annotated-spec/blob/master/phase0/beacon-chain.md#beaconstate)
    /// or [`historical_summaries`](https://github.com/ethereum/annotated-spec/blob/master/capella/beacon-chain.md#beaconstate) list.
    /// See the discussion [here](https://github.com/ethereum/annotated-spec/blob/master/phase0/beacon-chain.md#slots_per_historical_root)
    /// for more details about the `historical_roots` and [here](https://github.com/ethereum/annotated-spec/blob/master/capella/beacon-chain.md#historicalsummary)
    /// about `historical_summaries`.
    pub fn compute_block_roots_proof(&self, index: usize) -> Result<Vec<H256>, Error> {
        // Construct the block_roots Merkle tree and generate the proof.
        let leaves = self.data.block_roots().to_vec();
        let tree = MerkleTree::create(&leaves, HISTORY_TREE_DEPTH);
        let (_, mut proof) = tree.generate_proof(index, HISTORY_TREE_DEPTH)?;

        // We are going to verify this proof using the HistoricalSummary root, the two children nodes are the block_roots tree root and that state_roots tree root.
        // So we append the state_roots tree root to the proof.
        let state_roots_root = self.data.state_roots().tree_hash_root();
        proof.push(state_roots_root);

        Ok(proof)
    }

    pub fn compute_block_roots_proof_only(&self, index: usize) -> Result<Vec<H256>, Error> {
        let leaves = self.data.block_roots().to_vec();
        let tree = MerkleTree::create(&leaves, HISTORY_TREE_DEPTH);
        let (_, proof) = tree.generate_proof(index, HISTORY_TREE_DEPTH)?;

        Ok(proof)
    }
}

// Construct the block_roots Merkle tree and generate the proof.
pub fn compute_block_roots_proof_only<E: EthSpec>(
    block_roots: &[H256],
    index: usize,
) -> Result<Vec<H256>, Error> {
    let leaves = block_roots;
    let tree = MerkleTree::create(leaves, HISTORY_TREE_DEPTH);
    let (_, proof) = tree.generate_proof(index, HISTORY_TREE_DEPTH).unwrap();

    Ok(proof)
}
