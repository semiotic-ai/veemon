// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Fetch and encode Beacon Blocks
//!
//! Demonstrates how to fetch a single block from Beacon Firehose, and how to
//! encode it to a DBIN stream and store it to the filesystem (like the ETH example).

use beacon_protos::Block as BeaconBlock;
use firehose_client::{Chain, FirehoseClient};
use flat_files_encoder::Encoder;
use std::fs::File;

#[tokio::main]
async fn main() {
    // Fetch a single beacon block
    let mut beacon_client = FirehoseClient::new(Chain::Beacon);
    // This is the slot number for the Beacon slot for the example
    const SLOT_NUM: u64 = 9881091;
    let start_block: u64 = SLOT_NUM;
    let count: usize = 5;

    let mut blocks: Vec<BeaconBlock> = Vec::with_capacity(count);
    for i in 0..count {
        let n = start_block + i as u64;
        let resp = beacon_client.fetch_block(n).await.unwrap().unwrap();
        let block = BeaconBlock::try_from(resp.into_inner()).unwrap();
        blocks.push(block);
    }

    // Encode all fetched blocks as a single DBIN stream (one frame per block)
    let encoder = Encoder::new_v1("BEA");

    let path = format!("/tmp/mainnet_eth_blocks_{}_{}.dbin", start_block, count);
    let mut f = File::create(&path).unwrap();
    encoder
        .encode_prost_blocks_to_writer(&mut f, blocks)
        .unwrap();

    println!("Wrote {}", path);
}
