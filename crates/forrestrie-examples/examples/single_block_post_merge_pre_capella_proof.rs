// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Proof for  single block to be part of an era of beacon blocks using the [`HistoricalBatch`].
//!
//! Notice that A [`HistoricalBatch`]` isn't an accumulator, it is a list of block_roots and state_roots
//! So each root in the [`HistoricalRootsAccumulator`] corresponds to hash_tree_root(historical_batch).
//! The batch is used to verify era against the accumulator. A block can be verified against an
//! [`HistoricalBatch`], hence chaining the proofs
use std::fs;

use ethportal_api::consensus::beacon_state::HistoricalBatch;

use ssz::Decode;
use trin_validation::{
    historical_roots_acc::HistoricalRootsAccumulator, merkle::proof::verify_merkle_proof,
};

#[tokio::main]
async fn main() {
    // Load a historical batch.
    // A historical batch has to be generated from beacon blocks or retrieved
    // from some source that already calculated these
    let bytes =
        fs::read("./crates/forrestrie-examples/assets/historical_batch-573-c847a969.ssz").unwrap();
    let hist_batch = HistoricalBatch::from_ssz_bytes(&bytes).unwrap();

    // check if my block_root is inside the HistoricalBatch

    // construct proof from historical batch
    // In this example a slot that is inside the `HistoricalBatch`
    // was picked: https://beaconcha.in/slot/4685828
    // NOTICE: we can also use the block roots themselves inside the the HistoricalBatch
    // to figure out the slot by using the beacon chain explorer, for example:
    // https://beaconcha.in/slot/58bbce808c399069fdd3e02e7906cd382ba8ffac8c1625a9d801ffa6a4120c98
    const EPOCH_SIZE: i32 = 8192;
    let slot = 4685828;
    let historical_root_index: i32 = slot % EPOCH_SIZE;
    let historical_roots_proof =
        hist_batch.build_block_root_proof((historical_root_index as u32).into());

    // just checking if the rot macthes
    let block_root = hist_batch.block_roots[historical_root_index as usize];

    // The historical root we are getting:
    println!("root: {:?}, index, {:?}", block_root, historical_root_index);

    // // verify the proof
    let hist_acc = HistoricalRootsAccumulator::default();
    let block_root_index = slot % EPOCH_SIZE;
    let gen_index = 2 * EPOCH_SIZE + block_root_index;
    let historical_root_index = slot / EPOCH_SIZE;
    let historical_root = hist_acc.historical_roots[historical_root_index as usize];

    let result = verify_merkle_proof(
        block_root,
        &historical_roots_proof,
        14,
        gen_index as usize,
        historical_root,
    );

    println!("result of verifying proof: {:?}", result);
}
