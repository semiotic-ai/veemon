//! This example demonstrates how to calculate the receipts root of a block and
//! compare it to the receipts root in the block header.
//!
use alloy_primitives::FixedBytes;
use firehose_client::client::{Chain, FirehoseClient};
use firehose_protos::EthBlock as Block;

const BLOCK_NUMBER: u64 = 20672593;

#[tokio::main]
async fn main() {
    let mut client = FirehoseClient::new(Chain::Ethereum);
    let response = client.fetch_block(BLOCK_NUMBER).await.unwrap().unwrap();
    let block = Block::try_from(response.into_inner()).unwrap();

    let calculated_receipts_root = block.calculate_receipt_root().unwrap();

    // Compare the calculated receipts root to the receipts root in the block header
    assert_eq!(
        FixedBytes::<32>::from_slice(block.header.as_ref().unwrap().receipt_root.as_slice()),
        calculated_receipts_root
    );

    println!("Receipts root matches calculated receipts root!");
}
