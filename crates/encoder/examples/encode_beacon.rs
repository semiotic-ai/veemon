// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Fetch Beacon Block
//!
//! Demonstrates how to fetch a single block from Beacon Firehose, and how to
//! encode it to a DBIN stream and store it to the filesystem (like the ETH example).

use beacon_protos::Block as BeaconBlock;
use firehose_client::{Chain, FirehoseClient};
use flat_files_encoder::Encoder;
use prost::Message;

#[tokio::main]
async fn main() {
    // Fetch a single beacon block
    let mut beacon_client = FirehoseClient::new(Chain::Beacon);
    // This is the slot number for the Beacon slot for the example
    const SLOT_NUM: u64 = 9881091;

    let response = beacon_client.fetch_block(SLOT_NUM).await.unwrap().unwrap();
    let block = BeaconBlock::try_from(response.into_inner()).unwrap();

    // Encode the beacon block as a DBIN stream and write to /tmp
    let payload = block.encode_to_vec();
    let encoder = Encoder::new_v1("BEA");
    let dbin = encoder.encode_blocks(std::iter::once(payload));
    let path = format!("/tmp/mainnet_beacon_block_{}.dbin", SLOT_NUM);
    std::fs::write(&path, dbin).expect("Failed to write DBIN to /tmp");

    println!("Wrote {}", path);
}
