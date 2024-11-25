// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Ethereum Block to Beacon Slot Lookup Example
//!
//! This example performs a binary search to find the corresponding Beacon chain
//! slot for a given Ethereum execution block number. The problem being addressed
//! is that, due to missed slots in the Beacon chain, Ethereum execution blocks
//! and Beacon chain slots are not always aligned. Therefore, finding the correct
//! Beacon slot that contains the execution block requires searching through
//! Beacon blocks until the execution block is located.
//!
//! ## Key Concepts
//!
//! - **Execution Block Number**: This refers to the Ethereum block number that
//!   we're trying to locate within the Beacon chain.
//! - **Beacon Slot Number**: The slot number in the Beacon chain that contains
//!   the corresponding Ethereum execution block.
//! - **Deneb Fork**: This is the Ethereum fork that the blocks in the example
//!   are from. We can imagine using `const` values to represent the start slot
//!   of the Deneb fork and other upgrades, as well as the offsets between Ethereum
//!   and Beacon block numbers at different known points along the chain.
//!
//! ## Approach
//!
//! The example uses a binary search algorithm to locate the Beacon slot that
//! contains the execution block. It starts with a search range defined by
//! `DENEB_START_SLOT` and an upper bound based on an estimated offset.
//! During each iteration of the search, the Beacon block is fetched, and its
//! execution payload is examined to check if it contains the target Ethereum
//! block number. The search range is adjusted based on the result of this
//! comparison until the correct Beacon slot is found.
//!

use firehose_client::{Chain, FirehoseClient};
use forrestrie::{
    beacon_state::ETHEREUM_BEACON_DENEB_OFFSET,
    beacon_v1::{block, Block as FirehoseBeaconBlock},
};
use std::cmp::Ordering::*;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

/// This block relates to the slot represented by [`BEACON_SLOT_NUMBER`].
/// The execution block is in the execution payload of the Beacon block in slot [`BEACON_SLOT_NUMBER`].
const EXECUTION_BLOCK_NUMBER: u64 = 20759937;
/// This slot is the slot of the Beacon block that contains the execution block with [`EXECUTION_BLOCK_NUMBER`].
#[allow(unused)]
const BEACON_SLOT_NUMBER: u64 = 9968872; // Beacon slot 9968872 pairs with Ethereum block 20759937.

const IMAGINARY_CURRENT_SLOT: u64 = 10_000_000;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut beacon_client = FirehoseClient::new(Chain::Beacon);

    let mut low = EXECUTION_BLOCK_NUMBER - ETHEREUM_BEACON_DENEB_OFFSET as u64;
    let mut high = IMAGINARY_CURRENT_SLOT;

    let mut guesses = 0;

    while low <= high {
        guesses += 1;

        let mid = low + (high - low) / 2;

        info!(guess = mid, "Current guess for Beacon slot");

        let response = beacon_client.fetch_block(mid).await.unwrap().unwrap();
        let block = FirehoseBeaconBlock::try_from(response.into_inner()).unwrap();

        let Some(block::Body::Deneb(body)) = &block.body else {
            panic!("Unsupported block version!");
        };

        let execution_payload = body.execution_payload.as_ref().unwrap();
        let block_number = execution_payload.block_number;

        match block_number.cmp(&EXECUTION_BLOCK_NUMBER) {
            Less => low = mid + 1,
            Greater => high = mid - 1,
            Equal => {
                info!(
                    beacon_slot = block.slot,
                    "Found matching Beacon block: {}!", block.slot
                );
                break;
            }
        }

        if high == low || high == low + 1 {
            if let Some(final_result) = try_final_fetches(low, high, &mut beacon_client).await {
                println!(
                    "Found final result: matching execution block at Beacon slot: {}",
                    final_result
                );
                break;
            }
        }
    }
    info!(guesses, "Guesses");
}

/// Helper function to fetch both `low` and `high` Beacon slots when binary search is down to two options
async fn try_final_fetches(low: u64, high: u64, client: &mut FirehoseClient) -> Option<u64> {
    for slot in &[low, high] {
        let response = client.fetch_block(*slot).await.unwrap().unwrap();

        let block = FirehoseBeaconBlock::try_from(response.into_inner()).unwrap();

        let Some(block::Body::Deneb(body)) = &block.body else {
            return None;
        };

        let execution_payload = body.execution_payload.as_ref().unwrap();

        if execution_payload.block_number == EXECUTION_BLOCK_NUMBER {
            return Some(block.slot);
        }
    }
    None
}
