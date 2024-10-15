//! # Historical and state roots proof
//!
//! This example demonstrates how to prove the inclusion of historical state roots in the beacon state.
use forrestrie::beacon_state::{HeadState, HISTORICAL_ROOTS_FIELD_INDEX, HISTORICAL_ROOTS_INDEX};
use merkle_proof::verify_merkle_proof;
use types::{light_client_update::CURRENT_SYNC_COMMITTEE_PROOF_LEN, MainnetEthSpec};

#[tokio::main]
async fn main() {
    let url = format!("https://www.lightclientdata.org/eth/v2/debug/beacon/states/head");
    println!("Requesting head state ... (this can take a while!)");
    let response = reqwest::get(url).await.unwrap();
    let mut state: HeadState<MainnetEthSpec> = if response.status().is_success() {
        let json_response: serde_json::Value = response.json().await.unwrap();
        serde_json::from_value(json_response).unwrap()
    } else {
        panic!("Request failed with status: {}", response.status());
    };

    let proof = state
        .compute_merkle_proof_for_historical_data(HISTORICAL_ROOTS_INDEX)
        .unwrap();

    let historical_roots_tree_hash_root = state.historical_roots_tree_hash_root();

    let depth = CURRENT_SYNC_COMMITTEE_PROOF_LEN;

    let state_root = state.state_root().unwrap();

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

    println!("historical state roots proof verified successfully");
}
