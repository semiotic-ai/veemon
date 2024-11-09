//! # Fetch Beacon Block
//!
//! Demonstrates how to fetch a single block from Beacon Firehose, using the `Fetch` API.

use firehose_client::{Chain, FirehoseClient};
use firehose_protos::EthBlock;
use forrestrie::beacon_v1::{block::Body, Block as BeaconBlock};

#[tokio::main]
async fn main() {
    // Show matching data from execution layer and beacon chain
    let mut execution_layer_client = FirehoseClient::new(Chain::Ethereum);

    let response = execution_layer_client
        .fetch_block(20672593)
        .await
        .unwrap()
        .unwrap();

    let block = EthBlock::try_from(response.into_inner()).unwrap();

    assert_eq!(block.number, 20672593);
    assert_eq!(
        format!("0x{}", hex::encode(block.hash)).as_str(),
        "0xea48ba1c8e38ea586239e9c5ec62949ddd79404c6006c099bb02a8b22ddd18e4"
    );

    let mut beacon_client = FirehoseClient::new(Chain::Beacon);
    // This is the slot number for the Beacon block we want to fetch, but right now
    // we don't have a way to map the block number of the execution block to the slot number
    // of the Beacon block.
    let response = beacon_client.fetch_block(9881091).await.unwrap().unwrap();
    let block = BeaconBlock::try_from(response.into_inner()).unwrap();

    assert_eq!(block.slot, 9881091);

    let body = block.body.as_ref().unwrap();

    match body {
        Body::Deneb(body) => {
            let execution_payload = body.execution_payload.as_ref().unwrap();

            let block_hash = &execution_payload.block_hash;

            assert_eq!(
                format!("0x{}", hex::encode(block_hash)).as_str(),
                "0xea48ba1c8e38ea586239e9c5ec62949ddd79404c6006c099bb02a8b22ddd18e4"
            );

            let block_number = execution_payload.block_number;

            assert_eq!(block_number, 20672593);
        }
        _ => unimplemented!(),
    };

    println!("fetch_beacon ran to completion!");
}
