//! # Beacon Block and Header Root Consistency
//!
//! In Ethereum's Beacon chain, the beacon block root and the block header root should match
//! for any given block, ensuring data integrity. This can be verified by computing the root
//! of a block and comparing it to the root of its header.
//!
//! For example, for slot 8786333, the block's root can be computed using the `canonical_root` method
//! from [`TreeHash`]:
//!
//! ```rust
//! let block_root = block.canonical_root();
//! ```
//!
//! Similarly, the block header root is derived as follows:
//!
//! ```rust
//! let block_header_root = block.block_header().tree_hash_root();
//! ```
//!
//! Both of these root hashes should be identical, indicating that the block's root hash
//! correctly represents the block header:
//!
//! ```rust
//! assert_eq!(block_root, block_header_root);
//! ```
//!
use firehose_client::client::{Chain, FirehoseClient};
use forrestrie::beacon_v1::Block as FirehoseBeaconBlock;
use tree_hash::TreeHash;
use types::{BeaconBlock, MainnetEthSpec};

#[tokio::main]
async fn main() {
    let mut beacon_client = FirehoseClient::new(Chain::Beacon);
    let response = beacon_client.fetch_block(8786333).await.unwrap().unwrap();
    let beacon_block = FirehoseBeaconBlock::try_from(response.into_inner()).unwrap();

    let lighthouse_beacon_block = BeaconBlock::<MainnetEthSpec>::try_from(beacon_block).unwrap();

    insta::assert_debug_snapshot!(lighthouse_beacon_block.slot(), @
            "Slot(8786333)");

    let block_root = lighthouse_beacon_block.canonical_root();

    // See, for example, https://beaconcha.in/slot/8786333 and https://beaconscan.com/slot/8786333
    insta::assert_debug_snapshot!(block_root, @"0x063d4cf1a4f85d228d9eae17a9ab7df8b13de51e7a1988342a901575cce79613");

    let block_header = lighthouse_beacon_block.block_header();
    let block_header_root = block_header.tree_hash_root();

    assert_eq!(block_root, block_header_root);

    // This is to show that block hash and block body hash are different.
    let body = lighthouse_beacon_block.body_deneb().unwrap();
    let body_hash = body.tree_hash_root();
    insta::assert_debug_snapshot!(body_hash, @"0xc15e821344ce5b201e2938248921743da8a07782168456929c8cef9f25a4cb02");

    println!("fetch_and_verify_block.rs done");
}
