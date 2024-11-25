//! Proof for beacon blocks from merge until capella
//!
use std::{env, fs};

use ethportal_api::{
    consensus::beacon_state::HistoricalBatch,
    types::execution::header_with_proof::{BeaconBlockProof, HistoricalRootsBlockProof},
};
use forrestrie::beacon_state::{
    HeadState, HISTORICAL_SUMMARIES_FIELD_INDEX, HISTORICAL_SUMMARIES_INDEX,
};
use merkle_proof::verify_merkle_proof;
use reth_primitives::revm_primitives::B256;
use snap::raw::Decoder;
use ssz::{Decode, DecodeError, Encode};
use ssz_types::FixedVector;
use types::{light_client_update::CURRENT_SYNC_COMMITTEE_PROOF_LEN, MainnetEthSpec};

#[tokio::main]
async fn main() {
    // Load a historical batch.

    let compressed =
        fs::read("./crates/forrestrie-examples/assets/historical_batch-573-c847a969.ssz")
            .expect("Cannot read test file");
    let mut decoder = Decoder::new();
    let decompressed = decoder
        .decompress_vec(&compressed)
        .expect("Decompression failed");
    let hist_batch =
        HistoricalBatch::from_ssz_bytes(&decompressed).expect("Deserialization failed");
    assert_eq!(decompressed, hist_batch.as_ssz_bytes());
    // construct proof from historical batch
    // let historical_roots_proof = hist_batch.build_block_root_proof(0);

    // // construct the proof
    // let proof = HistoricalRootsBlockProof {
    //     beacon_block_proof: FixedVector::from_elem(B256::default()),
    //     beacon_block_root: B256::default(),
    //     historical_roots_proof: FixedVector::from_elem(B256::default()),
    //     slot: 0,
    // };
}
