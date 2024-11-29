// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! In this example, we verify a complete era of beacon blocks to be canonical.
//! We first read from storage a complete era of beacon blocks (8192 beacon blocks), compute the associated historical summary and
//! compare it against the historical summary from a current consensus stated.

use std::{fs::File, io::Read};

use forrestrie::beacon_state::{
    compute_block_roots_proof_only, HeadState, CAPELLA_START_ERA, HISTORY_TREE_DEPTH,
    SLOTS_PER_HISTORICAL_ROOT,
};
use primitive_types::H256;
use trin_validation::constants::EPOCH_SIZE;
use types::{historical_summary::HistoricalSummary, MainnetEthSpec};

use merkle_proof::verify_merkle_proof;
use merkle_proof::MerkleTree;

/// This slot is the starting slot of the Beacon block era.
const BEACON_SLOT_NUMBER: u64 = 10436608;

#[tokio::main]
async fn main() {
    println!("this also takes a while because the HeadState is a 700mb file");
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Get the head state of the Beacon chain from deserializing storage.
    let mut file = File::open("head_state.json").expect("Failed to open file");
    let mut json_string = String::new();
    file.read_to_string(&mut json_string)
        .expect("Failed to read file");

    let head_state: HeadState<MainnetEthSpec> =
        serde_json::from_str(&json_string).expect("Failed to deserialize head_state from JSON");

    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // get the block roots from deserializing storage.
    let mut file = File::open("beacon_block_roots.json").expect("Failed to open file");
    let mut json_string = String::new();
    file.read_to_string(&mut json_string)
        .expect("Failed to read file");

    let beacon_block_roots: Vec<H256> =
        serde_json::from_str(&json_string).expect("Failed to deserialize head_state from JSON");

    let era = BEACON_SLOT_NUMBER as usize / SLOTS_PER_HISTORICAL_ROOT;
    let era_index = era - CAPELLA_START_ERA;

    // Caculate the block_summary_root from the beacon blocks. Note that the block_summary_root is a field in the HistoricalSummary.
    let beacon_block_roots_tree_hash_root =
        MerkleTree::create(&beacon_block_roots, HISTORY_TREE_DEPTH).hash();

    let historical_summary: &HistoricalSummary = head_state
        .data()
        .historical_summaries()
        .unwrap()
        .get(era_index)
        .unwrap();

    let block_summary_root = historical_summary.block_summary_root();
    println!(
        "{:?}, {:?}",
        beacon_block_roots_tree_hash_root, block_summary_root
    );

    // if the roots match, it means that the whole era is included in the historical_summary
    assert_eq!(beacon_block_roots_tree_hash_root, block_summary_root);

    // It is also possible to make a verifiable merkle proof for it

    let index = BEACON_SLOT_NUMBER as usize % SLOTS_PER_HISTORICAL_ROOT as usize;
    let proof =
        compute_block_roots_proof_only::<MainnetEthSpec>(&beacon_block_roots, index).unwrap();

    let block_roots_tree_hash_root = historical_summary.block_summary_root();
    assert_eq!(proof.len(), HISTORY_TREE_DEPTH);

    // Verify the proof for a single block
    let historical_root_index: i32 = (BEACON_SLOT_NUMBER as usize % EPOCH_SIZE as usize)
        .try_into()
        .unwrap();
    let block_root = beacon_block_roots[historical_root_index as usize];

    assert!(
        verify_merkle_proof(
            // This is the equivalent beacon root of the slot in BEACON_SLOT_NUMBER constant
            block_root,
            &proof,             // the proof of the block's inclusion in the block roots
            HISTORY_TREE_DEPTH, // the depth of the block roots tree
            index,              // the index of the block in the era
            block_roots_tree_hash_root  // The root of the block roots
        ),
        "Merkle proof verification failed"
    );

    println!("successfully verified merkle proof")
}
