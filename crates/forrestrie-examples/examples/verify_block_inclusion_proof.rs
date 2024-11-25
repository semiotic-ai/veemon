// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Verify Block Inclusion Proof
//!
//! In Ethereum's Beacon Chain, execution layer payloads are included in the block body.
//!
//! This example demonstrates how to verify the inclusion proof of an execution payload
//! in a block body.
//!
//! For example, for block `20672593`, the execution payload root can be computed using the `tree_hash_root` method
//! from [`TreeHash`]:
//!
//! ```rust
//! let execution_payload_root = execution_payload.tree_hash_root();
//! ```
//!
//! Similarly, the block body root is derived as follows:
//!
//! ```rust
//! let block_body_hash = block_body.tree_hash_root();
//! ```
//!
//! The inclusion proof can be computed using the `compute_merkle_proof` method from [`BeaconBlockBody`]:
//!
//! ```rust
//! let proof = body.compute_merkle_proof(EXECUTION_PAYLOAD_INDEX).unwrap();
//! ```
//!
use firehose_client::{Chain, FirehoseClient};
use forrestrie::{
    beacon_block::{
        HistoricalDataProofs, BEACON_BLOCK_BODY_PROOF_DEPTH, EXECUTION_PAYLOAD_FIELD_INDEX,
    },
    beacon_v1::Block as FirehoseBeaconBlock,
};
use merkle_proof::verify_merkle_proof;
use tree_hash::TreeHash;
use types::{
    light_client_update::EXECUTION_PAYLOAD_INDEX, BeaconBlock, BeaconBlockBody, MainnetEthSpec,
};

#[tokio::main]
async fn main() {
    // test_inclusion_proof_for_block_body_given_execution_payload
    let mut beacon_client = FirehoseClient::new(Chain::Beacon);

    let response = beacon_client.fetch_block(20672593).await.unwrap().unwrap();

    let block = FirehoseBeaconBlock::try_from(response.into_inner()).unwrap();

    let beacon_block = BeaconBlock::<MainnetEthSpec>::try_from(block).unwrap();

    let execution_payload = beacon_block.body().execution_payload().unwrap();
    let execution_payload_root = execution_payload.tree_hash_root();

    let block_body = beacon_block.body_deneb().unwrap();
    let block_body_hash = block_body.tree_hash_root();

    let body = BeaconBlockBody::from(block_body.clone());
    let proof = body.compute_merkle_proof(EXECUTION_PAYLOAD_INDEX).unwrap();

    let depth = BEACON_BLOCK_BODY_PROOF_DEPTH;

    assert_eq!(proof.len(), depth, "proof length should equal depth");

    assert!(verify_merkle_proof(
        execution_payload_root,
        &proof,
        depth,
        EXECUTION_PAYLOAD_FIELD_INDEX,
        block_body_hash
    ));

    println!("verify_block_inclusion_proof.rs done");
}
