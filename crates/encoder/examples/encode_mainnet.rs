// SPDX-FileCopyrightText: 2024- Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0
//! # Fetch and encode Entire Era of Execution Layer Blocks
//!
//! This example demonstrates how to fetch an entire era of execution layer blocks
//! using the FirehoseClient.
use firehose_client::{Chain, FirehoseClient};
use firehose_protos::EthBlock;
use flat_files_encoder::Encoder;
use std::fs::File;

#[tokio::main]
async fn main() {
    let mut eth_client = FirehoseClient::new(Chain::Ethereum);

    let start_block: u64 = 12965000;
    let count: usize = 5;

    let mut blocks: Vec<EthBlock> = Vec::with_capacity(count);
    for i in 0..count {
        let n = start_block + i as u64;
        let resp = eth_client.fetch_block(n).await.unwrap().unwrap();
        let block = EthBlock::try_from(resp.into_inner()).unwrap();

        blocks.push(block);
    }

    // Encode all fetched blocks as a single DBIN stream (one frame per block)
    let encoder = Encoder::new_v1("ETH");

    let path = format!("/tmp/mainnet_eth_blocks_{}_{}.dbin", start_block, count);
    let mut f = File::create(&path).unwrap();
    encoder
        .encode_prost_blocks_to_writer(&mut f, blocks)
        .unwrap();

    println!("Wrote {}", path);
}
