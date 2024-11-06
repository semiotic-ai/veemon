//! # Fetch Ethereum Block
//!
//! Demonstrates how to fetch a single block from Ethereum firehose.

use firehose_client::client::{Chain, FirehoseClient};
use firehose_protos::EthBlock as Block;

#[tokio::main]
async fn main() {
    let mut client = FirehoseClient::new(Chain::Ethereum);
    let response = client.fetch_block(20672593).await.unwrap().unwrap();
    let block = Block::try_from(response.into_inner()).unwrap();

    assert_eq!(block.number, 20672593);
    assert_eq!(
        format!("0x{}", hex::encode(block.hash)).as_str(),
        "0xea48ba1c8e38ea586239e9c5ec62949ddd79404c6006c099bb02a8b22ddd18e4"
    );

    println!("fetch_beacon completed successfully!");
}
