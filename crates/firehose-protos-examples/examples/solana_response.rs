// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use firehose_client::{Chain, FirehoseClient};
use firehose_protos::SolBlock as Block;

const BLOCK_NUMBER: u64 = 333504000;

#[tokio::main]
async fn main() {
    let mut client = FirehoseClient::new(Chain::Solana);
    let response = client.fetch_block(BLOCK_NUMBER).await.unwrap().unwrap();
    let block = Block::try_from(response.into_inner()).unwrap();
    let block_slot = block.slot;
    println!("Slot number: {:?}", block_slot);
}
