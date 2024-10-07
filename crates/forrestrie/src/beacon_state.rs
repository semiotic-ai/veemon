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

/// The maximum number of block roots that can be stored in a `BeaconState`'s `block_roots` list.
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
    use std::{
        cell::LazyCell,
        collections::{BTreeMap, HashSet},
        sync::Mutex,
    };

    use super::*;

    use merkle_proof::verify_merkle_proof;
    use types::{light_client_update::CURRENT_SYNC_COMMITTEE_PROOF_LEN, MainnetEthSpec};

    // State for slot number 9471054, Deneb, latest execution payload header block number 20264676.
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

        // There are 8192 slots in an era. 8790016 / 8192 = 1073.
        let proof_era = transition_state.data().slot().as_usize() / 8192usize;

        // In this test we are using the historical_summaries (introduced in Capella) for verification,
        // so we need to subtract the Capella start era to get the correct index.
        let proof_era_index = proof_era - CAPELLA_START_ERA - 1;

        // We are going to prove that the block_root at index 4096 is included in the block_roots tree.
        // This is an arbitrary choice just for test purposes.
        let index = 4096usize;

        // Buffer of most recent 8192 block roots:
        let block_root_at_index = transition_state.data().block_roots().get(index).unwrap();

        let proof = transition_state.compute_block_roots_proof(index).unwrap();

        // To verify the proof, we use the state from a later slot.
        // The HistoricalSummary used to generate this proof is included in the historical_summaries list of this state.
        let state = &STATE;

        let state_lock = state.lock().unwrap();

        // The verifier retrieves the block_summary_root for the historical_summary and verifies the proof against it.
        let historical_summary = state_lock
            .data()
            .historical_summaries()
            .unwrap()
            .get(proof_era_index)
            .unwrap();

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

    #[test]
    fn test_empty_slot_block_hashes_are_duplicates_of_previous_full_slot_block() {
        // TODO(TRU-322): Test artifacts are a mess, ideally we would trim these JSON files
        // to only include the necessary fields.
        //
        // The JSON artifact used in this test is a BeaconState at slot 10035200, acquired from the
        // https://www.lightclientdata.org/eth/v2/debug/beacon/states/ provider.
        //
        // This slot was chosen because it is the first slot of an era (and an epoch),
        // which we demonstrate by showing that the slot number (see below) modulo 8192 is 0.
        let state: HeadState<MainnetEthSpec> =
            serde_json::from_str(std::include_str!("../../../state-10035200.json")).unwrap();

        let slot = state.data().slot().as_u64();
        insta::assert_debug_snapshot!(slot, @
            "10035200");

        // Slot 10035200 is the first slot of the 1225th era.
        // Every 8192 slots, the era increments by 1, and (after Capella) the historical summaries buffer is updated.
        let current_era = slot / 8192;
        assert_eq!(current_era, 1225);
        // Total number of slots at end of 1224th era is 10035200.
        assert_eq!(current_era * 8192, 10035200);
        // Remember, slots are counted using zero-based numbering.
        assert_eq!(slot % 8192, 0);

        // The historical summaries buffer is updated every 8192 slots, from the start of the Capella era.
        let num_historical_summaries = state.data().historical_summaries().unwrap().len() as u64;
        assert_eq!(
            (current_era - num_historical_summaries) as usize,
            CAPELLA_START_ERA
        );

        let block_roots = state.data().block_roots().to_vec();

        // Block roots buffer contains duplicates.
        let block_roots_set: HashSet<&H256, std::hash::RandomState> =
            HashSet::from_iter(block_roots.iter());
        assert_ne!(block_roots_set.len(), block_roots.len());

        let duplicate_block_roots_lookup_table = state
            .data()
            .block_roots()
            .to_vec()
            .iter()
            .enumerate()
            // Using BTreeMaps for deterministic order.
            .fold(BTreeMap::<H256, Vec<usize>>::new(), |mut acc, (i, root)| {
                acc.entry(*root).or_insert(Vec::new()).push(i);
                acc
            })
            // Remove non-duplicate block roots.
            .into_iter()
            .filter(|(_, indices)| indices.len() > 1)
            .collect::<BTreeMap<H256, Vec<usize>>>();

        insta::assert_debug_snapshot!(duplicate_block_roots_lookup_table, @r###"
        {
            0x0a82fc1f6bf1b23143f85193290ae6a1a3829f97b8cbebc7310ccd4e2670ac04: [
                8159,
                8160,
            ],
            0x10160a60710f5cd535744af4ab93945b53a69d8ad5c2d185cef39fb5db2c739f: [
                4829,
                4830,
            ],
            0x144136a34d785b87f9b10252d656b7d76d72bf24d6c98f1ecf3db075e65ba11f: [
                5408,
                5409,
            ],
            0x1f44d980ff3fad59550d67a72899c2a9a382c27dcac285da4a30f4813a32ec6b: [
                2082,
                2083,
            ],
            0x33efdf2390b28e8b83a7ce932e9f7ae652cc94baea7529b066406d92a17a0085: [
                2863,
                2864,
            ],
            0x389f6d0adc7c3ac69e335bd7a23bd021bb2ca7f0379cf4747b3a57b8a3e84c26: [
                6988,
                6989,
            ],
            0x512b89995f45b2518c1533d0d0c2868952ffee057520ca9c8abaa4583a755d0c: [
                3910,
                3911,
            ],
            0x6205798f4257b000255c8decdfc19eb0ab5a7b1a37007ca38af772d9c06d3663: [
                7263,
                7264,
            ],
            0x845cb1a3a04d8461ba50186d3c441316bf62530e3875bb95b342cb3f8a527d3b: [
                2568,
                2569,
            ],
            0x8748739e4d1c526616e9ad02669abb7c937cb9329e4ec43a4b6da7fa987e9ea5: [
                3776,
                3777,
                3778,
            ],
            0x9476f0da6e2711039adc15848cb2613c4c557f6dbc17b0ca6c2253b7b1583fb0: [
                7235,
                7236,
            ],
            0xa02ea51e375ce6d23bdbd6826ff9e9919b65d85aacffe241554e071e72fe55cc: [
                6214,
                6215,
            ],
            0xb4808b92f60e4261bd183000ace249bfe104f57f2fe4072c23966676defe5cd0: [
                7098,
                7099,
            ],
            0xb5aa2838677f7781c02eeed2503fe832a8193821568d91749614a417b9aefcce: [
                7383,
                7384,
            ],
            0xb84c76d1ec6022a00d445bd978445c5234c18ea33107121c622a75e2e4e59301: [
                3743,
                3744,
                3745,
            ],
            0xb99242051a8ba08e6b903b6e0b13b97d0ba1e81f805ead71b9f9ee0d0c50bc51: [
                8027,
                8028,
            ],
            0xc165e4bfbfa8aaccbc18030442e93db86062ed37564c52e47a54c6d65fdbeb71: [
                35,
                36,
            ],
            0xc90431f6b954f062c250f9146febc938d9e285e4f3c716512c3bcc57b0afcd14: [
                427,
                428,
            ],
            0xc9465a42c903868b90c1520abf25940a7d441e0b580e880b9873ef347a4cdfd2: [
                6852,
                6853,
            ],
            0xccc381c7dd2f4867f6c1b57970fffc143acf3b78219d18f39c511099ec240c8d: [
                1615,
                1616,
            ],
            0xcfab9ef6f4de3ef63a869ba093400a18a5cf49f16965d22dad820f107fc626a1: [
                6082,
                6083,
            ],
            0xd5981b2a671bce2bba148ba165637700f71c9dd1841c4f34622c04fda51449f9: [
                1839,
                1840,
            ],
            0xd7601a5fe3e02182a18a63b78b99327b63c915461d2b5c069203c5654c461521: [
                248,
                249,
            ],
            0xdbc617a37da7178961ed67da921ae4fd2b1d5d88965f1917bd498681f3964402: [
                5835,
                5836,
            ],
            0xe727e8acedd15bd499acf5f049c13c225d33a53a6281fcfce721f18ce5008103: [
                7405,
                7406,
            ],
            0xf07c053f494307167b4301b68739c87b57f55ae6417ca6727a9c32ad1d497555: [
                3579,
                3580,
            ],
            0xf355793a27a764dd9536b7bd8199ce73af90cd130edeb70f461f31fc3ce65fb4: [
                4530,
                4531,
            ],
            0xfc5647ebb69a2b32742387a2eda5b4ce0b8670721c8b4558477de554655fb0ec: [
                4174,
                4175,
            ],
        }
        "###);
    }
}
