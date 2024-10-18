//! # Prove Inclusion of a Single Execution Layer Block in the Canonical History of the Blockchain
//!
//! This example demonstrates how to prove the inclusion of a single execution layer block in the canonical
//! history of the blockchain.
//!
//! This method includes the following proofs:
//! 1. The block hash of the Ethereum block matches the hash in the block header.
//! 2. The block is the block it says it is, calculating its tree hash root matches the hash in the `root`
//!    field of the block.
//! 3. The Beacon block's Execution Payload matches the Ethereum block.
//! 4. Reproducing the block root for the "era", or 8192 slots, of a Beacon block's slot by streaming 8192
//!    Beacon blocks from the Beacon chain.
//! 5. Calculating the merkle proof of the block's inclusion in the block roots of the historical summary for
//!    the given era.
//!
//! We use a fork of [`lighthouse`](https://github.com/sigp/lighthouse)'s [`types`] that allows us to access
//! the `block_summary_root` of the [`HistoricalSummary`].
//!
//! While this example demonstrates verifying block 20759937 on the execution layer, you could also use the
//! same method to prove the inclusion of an entire era of 8192 blocks, since the method for verifying a single
//! block already includes streaming 8192 blocks for the era. And its the same 8192 blocks required to compute
//! the block roots tree hash root, which can then be compared to the tree hash root in the historical summary
//! for the era.
//!
use ethportal_api::Header;
use firehose_client::client::{Chain, FirehoseClient};
use forrestrie::{
    beacon_block::{
        HistoricalDataProofs, BEACON_BLOCK_BODY_PROOF_DEPTH, EXECUTION_PAYLOAD_FIELD_INDEX,
    },
    beacon_state::{
        compute_block_roots_proof_only, HeadState, CAPELLA_START_ERA, HISTORY_TREE_DEPTH,
        SLOTS_PER_HISTORICAL_ROOT,
    },
    BlockRoot,
};
use futures::StreamExt;
use merkle_proof::verify_merkle_proof;
use sf_protos::{beacon, ethereum};
use tree_hash::TreeHash;
use types::{
    historical_summary::HistoricalSummary, light_client_update::EXECUTION_PAYLOAD_INDEX,
    BeaconBlock, BeaconBlockBody, BeaconBlockBodyDeneb, ExecPayload, Hash256, MainnetEthSpec,
};

/// This block relates to the slot represented by [`BEACON_SLOT_NUMBER`].
/// The execution block is in the execution payload of the Beacon block in slot [`BEACON_SLOT_NUMBER`].
const EXECUTION_BLOCK_NUMBER: u64 = 20759937;
/// This slot is the slot of the Beacon block that contains the execution block with [`EXECUTION_BLOCK_NUMBER`].
const BEACON_SLOT_NUMBER: u64 = 9968872; // <- 9968872 pairs with 20759937

#[tokio::main]
async fn main() {
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Get the head state of the Beacon chain from a Beacon API provider.
    let state_handle = tokio::spawn(async move {
        let url = "https://www.lightclientdata.org/eth/v2/debug/beacon/states/head".to_string();
        println!("Requesting head state ... (this can take a while!)");
        let response = reqwest::get(url).await.unwrap();
        let head_state: HeadState<MainnetEthSpec> = if response.status().is_success() {
            let json_response: serde_json::Value = response.json().await.unwrap();
            serde_json::from_value(json_response).unwrap()
        } else {
            panic!("Request failed with status: {}", response.status());
        };
        head_state
    });

    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Get the Ethereum block.
    let mut eth1_client = FirehoseClient::new(Chain::Ethereum);
    let response = eth1_client
        .fetch_block(EXECUTION_BLOCK_NUMBER)
        .await
        .unwrap()
        .unwrap();
    let eth1_block = ethereum::r#type::v2::Block::try_from(response.into_inner()).unwrap();

    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // And get the Beacon block.
    let mut beacon_client = FirehoseClient::new(Chain::Beacon);
    let response = beacon_client
        .fetch_block(BEACON_SLOT_NUMBER)
        .await
        .unwrap()
        .unwrap();
    let beacon_block = beacon::r#type::v1::Block::try_from(response.into_inner()).unwrap();
    assert_eq!(beacon_block.slot, BEACON_SLOT_NUMBER);

    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Confirm that the block hash of the Ethereum block matches the hash in the block header.
    let block_header = Header::try_from(&eth1_block).unwrap();
    let eth1_block_hash = block_header.hash();
    assert_eq!(eth1_block_hash.as_slice(), &eth1_block.hash);

    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Convert the Beacon block to a Lighthouse `BeaconBlock`. This allows us to use Lighthouse's
    // implementation of the `TreeHash` trait to calculate the root of the Beacon block, which we
    // use to verify that the block is the block it says it is, i.e., the hash value in the `root`
    // field of the block matches the calculated root, which we calculate using the method implemented
    // on the `BeaconBlock` struct in Lighthouse.
    let lighthouse_beacon_block = BeaconBlock::<MainnetEthSpec>::try_from(beacon_block.clone())
        .expect("Failed to convert Beacon block to Lighthouse BeaconBlock");

    // Check the root of the Beacon block. This check shows that the calculation of the block root
    // of the Beacon block matches the hash in the `root` field of the block we fetched over gRPC;
    // the block is the block that it says it is.
    let lighthouse_beacon_block_root = lighthouse_beacon_block.canonical_root();
    assert_eq!(
        lighthouse_beacon_block_root.as_bytes(),
        beacon_block.root.as_slice()
    );
    let Some(beacon::r#type::v1::block::Body::Deneb(body)) = beacon_block.body else {
        panic!("Unsupported block version!");
    };
    let block_body: BeaconBlockBodyDeneb<MainnetEthSpec> = body.try_into().unwrap();

    // Confirm that the Beacon block's Execution Payload matches the Ethereum block we fetched.
    assert_eq!(
        block_body.execution_payload.block_number(),
        EXECUTION_BLOCK_NUMBER
    );

    // Confirm that the Ethereum block matches the Beacon block's Execution Payload.
    assert_eq!(
        block_body
            .execution_payload
            .block_hash()
            .into_root()
            .as_bytes(),
        eth1_block_hash.as_slice()
    );

    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Confirm that the Execution Payload is included in the Beacon block.
    let block_body_hash = block_body.tree_hash_root();
    let execution_payload = &block_body.execution_payload;
    let execution_payload_root = execution_payload.tree_hash_root();
    let body = BeaconBlockBody::from(block_body.clone());
    let proof = body.compute_merkle_proof(EXECUTION_PAYLOAD_INDEX).unwrap();
    let depth = BEACON_BLOCK_BODY_PROOF_DEPTH;
    assert!(verify_merkle_proof(
        execution_payload_root,
        &proof,
        depth,
        EXECUTION_PAYLOAD_FIELD_INDEX,
        block_body_hash
    ));

    // The era of the block's slot.
    // This is also the index of the historical summary containing the block roots for this era.
    let era = lighthouse_beacon_block.slot().as_usize() / SLOTS_PER_HISTORICAL_ROOT;

    println!("Requesting 8192 blocks for the era... (this takes a while)");
    let num_blocks = SLOTS_PER_HISTORICAL_ROOT as u64;
    let mut stream = beacon_client
        .stream_beacon_with_retry((era * SLOTS_PER_HISTORICAL_ROOT) as u64, num_blocks)
        .await
        .unwrap();
    let mut block_roots: Vec<Hash256> = Vec::with_capacity(SLOTS_PER_HISTORICAL_ROOT);
    while let Some(block) = stream.next().await {
        let root = BlockRoot::try_from(block).unwrap();
        block_roots.push(root.0);
    }
    assert_eq!(block_roots.len(), SLOTS_PER_HISTORICAL_ROOT);

    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // The index of the block in the complete era of block roots.
    // Beacon chain slot numbers are zero-based; genesis slot is 0.
    // We need this to calculate the merkle inclusion proof later.
    // If this is the first/last block of the era, the index is 0/8191.
    let index = lighthouse_beacon_block.slot().as_usize() % SLOTS_PER_HISTORICAL_ROOT;
    // Compute the proof of the block's inclusion in the block roots.
    let proof = compute_block_roots_proof_only::<MainnetEthSpec>(&block_roots, index).unwrap();
    // To get the correct index, we need to subtract the Capella start era.
    // `HistoricalSummary` was introduced in Capella and the block we're proving inclusion for is in
    // the post-Capella era.
    // For pre-Capella states, we would use the same method, only using the historical_roots field.
    let proof_era = era - CAPELLA_START_ERA;

    let head_state = state_handle.await.unwrap();
    let historical_summary: &HistoricalSummary = head_state
        .data()
        .historical_summaries()
        .unwrap()
        .get(proof_era)
        .unwrap();
    let block_roots_tree_hash_root = historical_summary.block_summary_root();
    assert_eq!(proof.len(), HISTORY_TREE_DEPTH);
    // Verify the proof.
    assert!(
        verify_merkle_proof(
            lighthouse_beacon_block_root, // the root of the block
            &proof,                       // the proof of the block's inclusion in the block roots
            HISTORY_TREE_DEPTH,           // the depth of the block roots tree
            index,                        // the index of the block in the era
            block_roots_tree_hash_root    // The root of the block roots
        ),
        "Merkle proof verification failed"
    );
    println!("All checks passed!");
}
