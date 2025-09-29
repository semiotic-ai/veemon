// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Example: Stream Ethereum Blocks
//!
//! This example demonstrates how to stream Ethereum blocks using the Firehose client.
use firehose_client::{Chain, FirehoseClient};
use futures::StreamExt;
use vee::EthBlock as Block;

#[tokio::main]
async fn main() {
    // Testing this so far without proper benchmarking, the time taken to fetch the blocks
    // grows linearly with the number of TOTAL_BLOCKS requested, to around 20 minutes for 8192 blocks!
    const TOTAL_BLOCKS: u64 = 100;
    const START_BLOCK: u64 = 19581798;

    let mut client = FirehoseClient::new(Chain::Ethereum);
    let mut stream = client
        .stream_blocks::<Block>(START_BLOCK, TOTAL_BLOCKS)
        .await
        .unwrap();

    let mut blocks: Vec<Block> = Vec::with_capacity(TOTAL_BLOCKS as usize);

    while let Some(block) = stream.next().await {
        blocks.push(block);
    }

    // For now, just using this to signal that the test has completed
    assert_eq!(blocks.len(), TOTAL_BLOCKS as usize);

    println!("stream_ethereum ran successfully");
}
