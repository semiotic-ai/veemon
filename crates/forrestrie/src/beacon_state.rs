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

pub const CAPELLA_START_ERA: usize = 758;

/// [`BeaconState`] `block_roots` vector has length `SLOTS_PER_HISTORICAL_ROOT` (See <https://github.com/ethereum/consensus-specs/blob/dev/specs/capella/beacon-chain.md#beaconstate>),
/// the value of which is calculated uint64(2**13) (= 8,192) (See <https://eth2book.info/capella/part3/config/preset/#time-parameters>)
pub const HISTORY_TREE_DEPTH: usize = 13;

/// The historical roots tree (pre-Capella) and the historical summaries tree (post-Capella) have the same depth.
/// Both tree's root has the block_roots tree root and the state_roots tree root as childen and so has one more layer than each of these trees.
pub const HISTORICAL_SUMMARY_TREE_DEPTH: usize = 14;

/// Historical roots is a top-level field on [`BeaconState`], subtract off the generalized indices
// for the internal nodes. Result should be 7, the field offset of the committee in the `BeaconState`:
// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#beaconstate
const HISTORICAL_ROOTS_INDEX: usize = 39;

/// Historical summaries is a top-level field on [`BeaconState`], subtract off the generalized indices
// for the internal nodes. Result should be 27, the field offset of the committee in the `BeaconState`:
// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#beaconstate
pub const HISTORICAL_SUMMARIES_INDEX: usize = 59;

/// Index of `historical_roots` field in the BeaconState [struct](https://github.com/ethereum/annotated-spec/blob/master/phase0/beacon-chain.md#beaconstate).
pub const HISTORICAL_ROOTS_FIELD_INDEX: usize = 7;

/// Index of `historical_summaries` field in the (post-Capella) BeaconState [struct](https://github.com/ethereum/annotated-spec/blob/master/capella/beacon-chain.md#beaconstate).
pub const HISTORICAL_SUMMARIES_FIELD_INDEX: usize = 27;

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
    /// Computes a Merkle inclusion proof of a `BeaconBlock` root using Merkle trees from either
    /// the [`historical_roots`](https://github.com/ethereum/annotated-spec/blob/master/phase0/beacon-chain.md#beaconstate)
    /// or [`historical_summaries`](https://github.com/ethereum/annotated-spec/blob/master/capella/beacon-chain.md#beaconstate) list.
    /// See the discussion [here](https://github.com/ethereum/annotated-spec/blob/master/phase0/beacon-chain.md#slots_per_historical_root)
    /// for more details about the `historical_roots` and [here](https://github.com/ethereum/annotated-spec/blob/master/capella/beacon-chain.md#historicalsummary)
    /// about `historical_summaries`.
    pub fn compute_block_roots_proof(&self, index: usize) -> Result<Vec<H256>, Error> {
        // This computation only makes sense if we have all of the leaves (BeaconBlock roots) to construct the HistoricalSummary Merkle tree.
        // So we construct a new HistoricalSummary from the state and check that the tree root is in historical_summaries.
        // This will only be true if the state is in the last slot of an era.
        let historical_summary = HistoricalSummary::new(&self.data);
        let historical_summaries = self.data.historical_summaries()?.to_vec();
        let latest_historical_summary = historical_summaries.last();
        if latest_historical_summary != Some(&historical_summary) {
            return Err(Error::SlotOutOfBounds);
        }

        // Construct the block_roots Merkle tree and generate the proof.
        let leaves = self.data.block_roots().to_vec();
        let tree = MerkleTree::create(&leaves, HISTORY_TREE_DEPTH);
        let (_, mut proof) = tree.generate_proof(index, HISTORY_TREE_DEPTH)?;

        // We are going to verify this proof using the HistoricalSummary root, the two children nodes are the block_roots tree root and that state_roots tree root.
        // So we append the state_roots tree root to the proof.
        let state_roots_root = self.data.state_roots().tree_hash_root();
        proof.extend(vec![state_roots_root]);

        Ok(proof)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::LazyCell, sync::Mutex};

    use super::*;

    use merkle_proof::verify_merkle_proof;
    use types::{light_client_update::CURRENT_SYNC_COMMITTEE_PROOF_LEN, MainnetEthSpec};

    const HEAD_STATE_JSON: &str = include_str!("../../../head-state.json");
    const STATE: LazyCell<Mutex<HeadState<MainnetEthSpec>>> = LazyCell::new(|| {
        Mutex::new({
            serde_json::from_str(HEAD_STATE_JSON).expect(
            "For this spike we are using a 'head-state.json' file that has been shared among contributors",
        )
        })
    });

    const TRANSITION_STATE_JSON: &str = include_str!("../../../8790016-state.json");
    const TRANSITION_STATE: LazyCell<HeadState<MainnetEthSpec>> = LazyCell::new(|| {
        serde_json::from_str(TRANSITION_STATE_JSON).expect(
            "For this spike we are using a '8790016-state.json' file that has been shared among contributors",
        )
    });

    #[test]
    fn test_inclusion_proofs_with_historical_and_state_roots() {
        let state = STATE;

        let state_lock = state.lock().unwrap();

        let proof = state_lock
            .compute_merkle_proof_for_historical_data(HISTORICAL_ROOTS_INDEX)
            .unwrap();

        drop(state_lock);

        insta::assert_debug_snapshot!(proof, @r###"
        [
            0xe81a79506c46b126f75a08cdd5cbc35052b61ca944c6c3becf32432e2ee6373a,
            0xcfb49cd7eb0051153685e5e6124b635c6b9bcc69a6ead6af0ef7d9885fcc16e2,
            0x29c2e1f6d96493e9b49517cb78123990038429e4c3574688a48f9abe69238449,
            0xdb329a01d9114f087155633b36b498c8e60028c0acedc8e3b64e013dbbd4fa06,
            0x53b107024e402f616f8f348d900e0d62f4b6f0558d2bfbd09200e68620a5b9c2,
        ]
        "###);

        let mut state_lock = state.lock().unwrap();

        let historical_roots_tree_hash_root = state_lock.historical_roots_tree_hash_root();

        let state_root = state_lock.state_root().unwrap();

        drop(state_lock);

        let depth = CURRENT_SYNC_COMMITTEE_PROOF_LEN;

        assert!(
            verify_merkle_proof(
                historical_roots_tree_hash_root,
                &proof,
                depth,
                HISTORICAL_ROOTS_FIELD_INDEX,
                state_root
            ),
            "Merkle proof verification failed"
        );
    }

    #[test]
    fn test_inclusion_proofs_for_historical_summary_given_historical_summaries_root() {
        let state = &STATE;

        let state_lock = state.lock().unwrap();

        let proof = state_lock
            .compute_merkle_proof_for_historical_data(HISTORICAL_SUMMARIES_INDEX)
            .unwrap();

        drop(state_lock);

        insta::assert_debug_snapshot!(proof, @r###"
        [
            0x053a090000000000000000000000000000000000000000000000000000000000,
            0x455a0d1e0a3b5660d74b6520062c9c3cead986928686e535451ca6e61aeb291f,
            0xdb56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71,
            0xc204e43766c4e9d43da1a54c3053024eef28d407bcca7936900ffd2e7aa165b2,
            0x2150a88f205759c59817f42dc307620c67d3d23417959286928d186c639a0948,
        ]
        "###);

        let mut state_lock = state.lock().unwrap();

        let historical_summaries_tree_hash_root =
            state_lock.historical_summaries_tree_hash_root().unwrap();

        let state_root = state_lock.state_root().unwrap();

        drop(state_lock);

        let depth = CURRENT_SYNC_COMMITTEE_PROOF_LEN;

        assert!(
            verify_merkle_proof(
                historical_summaries_tree_hash_root,
                &proof,
                depth,
                HISTORICAL_SUMMARIES_FIELD_INDEX,
                state_root
            ),
            "Merkle proof verification failed"
        );
    }

    #[test]
    /// For this test, we want to prove that a block_root is included in a HistoricalSummary from the BeaconState historical_summaries List.
    /// A HistoricalSummary contains the roots of two Merkle trees, block_summary_root and state_summary root.
    /// We are interested in the block_summary tree, whose leaves consists of the BeaconBlockHeader roots for one epoch (8192 consecutive slots).  
    /// For this test, we are using the state at slot 8790016, which is the last slot of epoch 1073, to build the proof.
    /// We chose this slot because it is the last slot of an epoch, and all of the BeaconBlockHeader roots needed to construct the
    /// HistoricalSummary for this epoch are available in state.block_roots.
    fn test_inclusion_proofs_for_block_roots() {
        let transition_state = &TRANSITION_STATE;

        // There are 8192 slots in an era.
        let proof_era = transition_state.data().slot().as_usize() / 8192usize;

        // In this test we are using the historical_summaries (introduced in Capella) for verification, so we need to subtract the Capella start era to get the correct index.
        let proof_era_index = proof_era - CAPELLA_START_ERA - 1;

        // We are going to prove that the block_root at index 4096 is included in the block_roots tree.
        // This is an arbitrary choice just for test purposes.
        let index = 4096usize;

        let block_root_at_index = match transition_state.data().block_roots().get(index) {
            Some(block_root) => block_root,
            None => panic!("Block root not found"),
        };

        let proof = match transition_state.compute_block_roots_proof(index) {
            Ok(proof) => proof,
            Err(e) => panic!("Error generating block_roots proof: {:?}", e),
        };

        // To verify the proof, we use the state from a later slot.
        // The HistoricalSummary used to generate this proof is included in the historical_summaries list of this state.
        let state = &STATE;

        let state_lock = state.lock().unwrap();

        // The verifier retrieves the block_summary_root for the historical_summary and verifies the proof against it.
        let historical_summary = match state_lock
            .data()
            .historical_summaries()
            .unwrap()
            .get(proof_era_index)
        {
            Some(historical_summary) => historical_summary,
            None => panic!("HistoricalSummary not found"),
        };

        let historical_summary_root = historical_summary.tree_hash_root();

        drop(state_lock);

        assert!(
            verify_merkle_proof(
                *block_root_at_index,
                &proof,
                HISTORICAL_SUMMARY_TREE_DEPTH,
                index,
                historical_summary_root
            ),
            "Merkle proof verification failed"
        );
    }
}
