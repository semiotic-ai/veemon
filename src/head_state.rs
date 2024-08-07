use merkle_proof::{MerkleTree, MerkleTreeError};
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use tree_hash::TreeHash;
use types::{BeaconState, BeaconStateError as Error, EthSpec, MainnetEthSpec, historical_summary::HistoricalSummary};

const HISTORY_TREE_DEPTH: usize = 13;
const HISTORICAL_SUMMARY_TREE_DEPTH: usize = 14;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HeadState<E: EthSpec> {
    version: String,
    execution_optimistic: bool,
    data: BeaconState<E>,
}

impl HeadState<MainnetEthSpec> {
    pub fn compute_merkle_proof(&self, index: usize) -> Result<Vec<H256>, Error> {
        self.data.compute_merkle_proof(index)
    }

    pub fn data(&self) -> &BeaconState<MainnetEthSpec> {
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

    pub fn state_root(&self) -> H256 {
        self.data.tree_hash_root()
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn compute_block_roots_proof(&self, index: usize) -> Result<Vec<H256>, Error> {
        // This computation only makes sense if we have all of the leaves (BeaconBlock roots) to construct the HisoricalSummary Merkle tree.
        // So we construct a new HistoricalSummary from the state and check that the tree root is in historical_summaries.
        // This will only be true if the state is in the last slot of an era.
        let historical_summary = HistoricalSummary::new(&self.data);
        let historical_summaries= self.data.historical_summaries()?.to_vec();
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
    use std::cell::LazyCell;

    use super::*;

    use merkle_proof::verify_merkle_proof;
    use types::light_client_update::{
        CURRENT_SYNC_COMMITTEE_PROOF_LEN, HISTORICAL_ROOTS_INDEX, HISTORICAL_SUMMARIES_INDEX,
    };

    const HEAD_STATE_JSON: &str = include_str!("../head-state.json");
    
    const HISTORICAL_ROOTS_FIELD_INDEX: usize = 7;
    const HISTORICAL_SUMMARIES_FIELD_INDEX: usize = 27;

    const STATE: LazyCell<HeadState<MainnetEthSpec>> = LazyCell::new(|| {
        serde_json::from_str(HEAD_STATE_JSON).expect(
            "For this spike we are using a 'head-state.json' file that has been shared among contributors",
        )
    });

    const TRANSITION_STATE_JSON: &str = include_str!("../8790016-state.json");
    const TRANSITION_STATE: LazyCell<HeadState<MainnetEthSpec>> = LazyCell::new(|| {
        serde_json::from_str(TRANSITION_STATE_JSON).expect(
            "For this spike we are using a '8790016-state.json' file that has been shared among contributors",
        )
    });

    const CAPELLA_START_ERA: usize = 758;

    #[test]
    fn test_inclusion_proofs_with_historical_and_state_roots() {
        let state = &STATE;

        let proof = state.compute_merkle_proof(HISTORICAL_ROOTS_INDEX).unwrap();

        insta::assert_debug_snapshot!(proof, @r###"
        [
            0xe81a79506c46b126f75a08cdd5cbc35052b61ca944c6c3becf32432e2ee6373a,
            0xcfb49cd7eb0051153685e5e6124b635c6b9bcc69a6ead6af0ef7d9885fcc16e2,
            0x29c2e1f6d96493e9b49517cb78123990038429e4c3574688a48f9abe69238449,
            0xdb329a01d9114f087155633b36b498c8e60028c0acedc8e3b64e013dbbd4fa06,
            0x53b107024e402f616f8f348d900e0d62f4b6f0558d2bfbd09200e68620a5b9c2,
        ]
        "###);

        let historical_roots_tree_hash_root = state.historical_roots_tree_hash_root();

        let state_root = state.state_root();

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

        let proof = state
            .compute_merkle_proof(HISTORICAL_SUMMARIES_INDEX)
            .unwrap();

        insta::assert_debug_snapshot!(proof, @r###"
        [
            0x053a090000000000000000000000000000000000000000000000000000000000,
            0x455a0d1e0a3b5660d74b6520062c9c3cead986928686e535451ca6e61aeb291f,
            0xdb56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71,
            0xc204e43766c4e9d43da1a54c3053024eef28d407bcca7936900ffd2e7aa165b2,
            0x2150a88f205759c59817f42dc307620c67d3d23417959286928d186c639a0948,
        ]
        "###);

        let historical_summaries_tree_hash_root =
            state.historical_summaries_tree_hash_root().unwrap();

        let state_root = state.state_root();

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
    fn test_inclusion_proofs_for_block_roots() {
        // For this test, we want to prove that a block_root is included in a HistoricalSummary from the BeaconState historical_summaries List.
        // A HistoricalSummary contains the roots of two Merkle trees, block_summary_root and state_summary root.
        // We are interested in the block_summary tree, whose leaves consists of the BeaconBlockHeader roots for one epoch (8192 consecutive slots).  
        // For this test, we are using the state at slot 8790016, which is the last slot of epoch 1073, to build the proof.
        // We chose this slot because it is the last slot of an epoch, and all of the BeaconBlockHeader roots needed to construct the HistoricalSummary for this epoch are available in state.block_roots. 
        let transition_state = &TRANSITION_STATE;
        
        // There are 8192 slots in an era.
        let proof_era= transition_state.data().slot().as_usize() / 8192usize;
        
        // In this test we are using the historical_summaries (introduced in Capella) for verification, so we need to subtract the Capella start era to get the correct index.
        let proof_era_index = proof_era - CAPELLA_START_ERA - 1;

        // We are going to prove that the block_root at index 4096 is included in the block_roots tree.
        // This is an arbitrary choice just for test purposes.
        let index = 4096usize;
        let block_root_at_index = match transition_state.data().block_roots().get(index) {
            Some(block_root) => block_root,
            None => panic!("Block root not found")
        };
        let proof = match transition_state.compute_block_roots_proof(index) {
            Ok(proof) => proof,
            Err(e) => panic!("Error generating block_roots proof: {:?}", e)
        };

        // To verify the proof, we use the state from a later slot.
        // The HistoricalSummary used to generate this proof is included in the historical_summaries list of this state.
        let state = &STATE;

        // The verifier retrieves the block_summary_root for the historical_summary and verifies the proof against it.
        let historical_summary= match state.data().historical_summaries().unwrap().get(proof_era_index){
            Some(historical_summary) => historical_summary,
            None => panic!("HistoricalSummary not found")
        };
        let historical_summary_root = historical_summary.tree_hash_root();
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
