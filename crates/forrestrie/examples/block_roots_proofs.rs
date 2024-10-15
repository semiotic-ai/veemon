//! # Block Roots Proofs
//!
//! We want to prove that a block_root is included in a [`HistoricalSummary`] from the [`BeaconState`] historical_summaries List.
//! A [`HistoricalSummary`] contains the roots of two Merkle trees, block_summary_root and state_summary root.
//! We are interested in the block_summary tree, whose leaves consists of the [`BeaconBlockHeader`] roots for one era (8192 consecutive slots).  
//! For example, we could have used the state at [`Slot`] 8790016, which is the first [`Slot`] of era 1073, to build the proof.
//! Because it is the first [`Slot`] of an era, all of the [`BeaconBlockHeader`] roots needed to construct the
//! [`HistoricalSummary`] for this era are available in state.block_roots.
//! The last block root in the `block_roots` buffer is the block root of the previous block.

use forrestrie::beacon_state::{
    HeadState, CAPELLA_START_ERA, HISTORICAL_SUMMARY_TREE_DEPTH, SLOTS_PER_HISTORICAL_ROOT,
};
use merkle_proof::verify_merkle_proof;
use tree_hash::TreeHash;
use types::MainnetEthSpec;
#[tokio::main]
async fn main() {
    // You may need to update the slot being queried as the state data is updated.
    // Multiply a recent era by 8192 to get the slot number.
    const SLOT: u64 = 10182656;
    let url = format!("https://www.lightclientdata.org/eth/v2/debug/beacon/states/{SLOT}");
    println!("Requesting state for slot {SLOT} ... (this can take a while!)");
    let response = reqwest::get(url).await.unwrap();
    let transition_state: HeadState<MainnetEthSpec> = if response.status().is_success() {
        let json_response: serde_json::Value = response.json().await.unwrap();
        serde_json::from_value(json_response).unwrap()
    } else {
        panic!("Request failed with status: {}", response.status());
    };

    // There are 8192 slots in an era.
    let proof_era = transition_state.data().slot().as_usize() / SLOTS_PER_HISTORICAL_ROOT;

    // In this test we are using the historical_summaries (introduced in Capella) for verification,
    // so we need to subtract the Capella start era to get the correct index.
    let proof_era_index = proof_era - CAPELLA_START_ERA - 1;

    // We are going to prove that the block_root at index 4096 is included in the block_roots tree.
    // This is an arbitrary choice just for test purposes.
    let index = 4096usize;

    // Buffer of most recent 8192 block roots:
    let block_root_at_index = *transition_state.data().block_roots().get(index).unwrap();

    assert!(
        transition_state.block_roots_contain_entire_era().unwrap(),
        "Block roots buffer does not contain an entire era"
    );

    let proof = transition_state.compute_block_roots_proof(index).unwrap();

    // To verify the proof, we use the state from a later slot.
    // The HistoricalSummary used to generate this proof is included in the historical_summaries list of this state.
    let url = "https://www.lightclientdata.org/eth/v2/debug/beacon/states/head".to_string();
    println!("Requesting head state ... (this can take a while!)");
    let response = reqwest::get(url).await.unwrap();
    let state: HeadState<MainnetEthSpec> = if response.status().is_success() {
        let json_response: serde_json::Value = response.json().await.unwrap();
        serde_json::from_value(json_response).unwrap()
    } else {
        panic!("Request failed with status: {}", response.status());
    };

    // The verifier retrieves the block_summary_root for the historical_summary and verifies the proof against it.
    let historical_summary = state
        .data()
        .historical_summaries()
        .unwrap()
        .get(proof_era_index)
        .unwrap();

    let historical_summary_root = historical_summary.tree_hash_root();

    assert!(
        verify_merkle_proof(
            block_root_at_index,
            &proof,
            HISTORICAL_SUMMARY_TREE_DEPTH,
            index,
            historical_summary_root
        ),
        "Merkle proof verification failed"
    );

    println!("Block roots proof verified successfully");
}
