//! Proof for an era of beacon blocks using the [`HistoricalBatch`].
//!
use std::{env, fs, str::FromStr};

use ethportal_api::{
    consensus::beacon_state::HistoricalBatch,
    types::execution::header_with_proof::HistoricalRootsBlockProof,
};

use reth_primitives::revm_primitives::{alloy_primitives::BlockHash, B256};
use ssz::Decode;
use ssz_types::FixedVector;
use trin_validation::{
    historical_roots_acc::HistoricalRootsAccumulator, merkle::proof::verify_merkle_proof,
};
use types::{light_client_update::CURRENT_SYNC_COMMITTEE_PROOF_LEN, MainnetEthSpec};

#[tokio::main]
async fn main() {
    // Load a historical batch.
    // A historical batch has to be generated from beacon blocks or retrieved
    // from some source that already calculated these
    let bytes =
        fs::read("./crates/forrestrie-examples/assets/historical_batch-573-c847a969.ssz").unwrap();
    let hist_batch = HistoricalBatch::from_ssz_bytes(&bytes).unwrap();
    // construct proof from historical batch
    let historical_roots_proof = hist_batch.build_block_root_proof(0);

    // // verify the proof
    let epoch_size = 8192;
    let slot = 4_698_112;
    let block_root_index = slot % epoch_size;
    let historical_root_index: i32 = slot / epoch_size;
    let hist_acc = HistoricalRootsAccumulator::default();
    let historical_root = hist_acc.historical_roots[historical_root_index as usize];

    let gen_index = 2 * epoch_size + block_root_index;

    let result = verify_merkle_proof(
        B256::from_str("0x5273538177993fb75d8d27a00f32cd6cf583755062e97a45eb362cac356e3088")
            .unwrap(),
        &historical_roots_proof,
        14,
        gen_index as usize,
        historical_root,
    );
    println!("result of verifying proof: {:?}", result);
}
