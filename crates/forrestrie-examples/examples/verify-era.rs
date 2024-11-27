//! In this example, we verify a complete era of both beacon blocks and execution blocks.
//! We first fetch a complete era of beacon blocks (8192 beacon blocks), compute the associated historical summary and
//! compare it against the historical summary from a current consensus stated. We also extract the
//! execution block headers and block numbers from the beacon blocks. We then fetch the execution
//! blocks using the extracted block numbers and verify the execution block data against the
//! extracted block headers.

use ethportal_api::Header;
use firehose_client::{Chain, FirehoseClient};
use firehose_protos::EthBlock;
use forrestrie::{
    beacon_state::{HeadState, CAPELLA_START_ERA, HISTORY_TREE_DEPTH, SLOTS_PER_HISTORICAL_ROOT},
    beacon_v1::{self},
};
use futures::StreamExt;
use tree_hash::TreeHash;
use types::{
    historical_summary::HistoricalSummary, BeaconBlock, BeaconBlockBodyDeneb, ExecPayload,
    MainnetEthSpec, Slot,
};

use merkle_proof::MerkleTree;

/// This slot is the starting slot of the Beacon block era.
const BEACON_SLOT_NUMBER: u64 = 10436608;
/// The URL to fetch the head state of the Beacon chain.
const LIGHT_CLIENT_DATA_URL: &str =
    "https://www.lightclientdata.org/eth/v2/debug/beacon/states/head";

#[tokio::main]
async fn main() {
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Get the head state of the Beacon chain from a Beacon API provider.
    let state_handle = tokio::spawn(async move {
        let url = LIGHT_CLIENT_DATA_URL.to_string();
        println!("Requesting head state ... (this can take a while!)");
        let response = reqwest::get(url).await.unwrap();
        let head_state: HeadState<MainnetEthSpec> = response.json().await.unwrap();
        head_state
    });

    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Here we are going to fetch all of the beacon blocks for an era.
    // We will verify that the blocks are correct by computing a block_summary_root from the beacon blocks and comparing it to the block_summary_root in the historical summary from the consensus state.
    let mut beacon_client = FirehoseClient::new(Chain::Beacon);

    // The era of the block's slot.
    // This is also the index of the historical summary containing the block roots for this era.
    let era = BEACON_SLOT_NUMBER as usize / SLOTS_PER_HISTORICAL_ROOT;

    // Stream the blocks
    println!("Requesting 8192 blocks for the era... (this takes a while)");
    let num_blocks = SLOTS_PER_HISTORICAL_ROOT as u64;
    let mut stream = beacon_client
        .stream_beacon_with_retry((era * SLOTS_PER_HISTORICAL_ROOT) as u64, num_blocks)
        .await
        .unwrap();

    // We are going to store off the execution block numbers and hashes for later verification.
    let mut execution_block_number_and_hash = Vec::with_capacity(SLOTS_PER_HISTORICAL_ROOT);

    // We are going to store off the beacon block roots and calculate the block_summary_root from them.
    let mut beacon_block_roots = Vec::with_capacity(SLOTS_PER_HISTORICAL_ROOT);

    let mut idx = 0;
    let mut prev_slot = Slot::new(0);
    let mut push_parent_root = false;
    while let Some(block) = stream.next().await {
        // Get the exeuction block number and blockhash.
        let lighthouse_beacon_block = BeaconBlock::<MainnetEthSpec>::try_from(block.clone())
            .expect("Failed to convert Beacon block to Lighthouse BeaconBlock");
        let Some(beacon_v1::block::Body::Deneb(body)) = block.body else {
            panic!("Unsupported block version!");
        };
        let block_body: BeaconBlockBodyDeneb<MainnetEthSpec> = body.try_into().unwrap();
        let execution_block_number = block_body.execution_payload.block_number();
        let execution_block_hash = block_body.execution_payload.block_hash();
        execution_block_number_and_hash.push((execution_block_number, execution_block_hash));

        // There are a few things going on here:
        // 1. there is currently a bug in the Firehose API where if a slot does not have an execution payload (the slot was skipped), then Firehose simply repeats the previous beacon block.
        // This is a problem because this means that we can't calculate the beacon block root for the skipped slot.
        // As a workaround, whenever we see a repeated block (implying a skipped slot), we will skip processing that block and on the next block we will push the parent root to the beacon block roots.
        // Assuming that the parent root is correct, then the block_summary_root will be correct.
        //
        // 2. We are going to check the consistency of the beacon chain by comparing the claimed parent root of the current block against the previous block's root, they should match.
        // This helps us catch errors within the era.

        if idx > 0 {
            // If there was a skipped slot, then we will skip processing the current block and push the parent root to the beacon block roots.
            let curr_slot = lighthouse_beacon_block.as_deneb().unwrap().slot;
            if curr_slot == prev_slot {
                push_parent_root = true;
                idx += 1;
                println!("Slot skipped!");
                continue;
            }
            if push_parent_root {
                let parent_root = lighthouse_beacon_block.as_deneb().unwrap().parent_root;
                beacon_block_roots.push(parent_root);
                push_parent_root = false;
            }

            // Check the parent root of the current block against the previous block's root.
            let prev_block_root = beacon_block_roots[idx - 1];
            let prev_block_root_from_block =
                lighthouse_beacon_block.as_deneb().unwrap().parent_root;
            if prev_block_root != prev_block_root_from_block {
                println!("Slot {}", lighthouse_beacon_block.as_deneb().unwrap().slot);
                panic!("Block root mismatch!");
            }
            println!(
                "Slot {} verified!",
                lighthouse_beacon_block.as_deneb().unwrap().slot
            );
        }

        // Store the beacon block root.
        let beacon_block_root = lighthouse_beacon_block.tree_hash_root();
        beacon_block_roots.push(beacon_block_root);
        idx += 1;
        prev_slot = lighthouse_beacon_block.as_deneb().unwrap().slot;
    }

    // Check that we have the correct number of blocks.
    assert_eq!(
        execution_block_number_and_hash.len(),
        SLOTS_PER_HISTORICAL_ROOT
    );
    assert_eq!(beacon_block_roots.len(), SLOTS_PER_HISTORICAL_ROOT);

    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Here is where we check that the historical summary from the consensus state matches the historical summary computed from the beacon blocks.

    // Caculate the block_summary_root from the beacon blocks. Note that the block_summary_root is a field in the HistoricalSummary.
    let beacon_block_roots_tree_hash_root =
        MerkleTree::create(&beacon_block_roots, HISTORY_TREE_DEPTH).hash();

    // To get the correct index for the era's HistoricalSummary in the consensus state, we need to subtract the Capella start era.
    // `HistoricalSummary` was introduced in Capella and the block we're proving inclusion for is in
    // the post-Capella era.
    // For pre-Capella states, we would use the same method, only using the historical_roots field.
    let era_index = era as usize - CAPELLA_START_ERA;

    // Get the historical summary for the era from the consensus state.
    let head_state = state_handle.await.unwrap();
    let historical_summary: &HistoricalSummary = head_state
        .data()
        .historical_summaries()
        .unwrap()
        .get(era_index)
        .unwrap();

    let block_summary_root = historical_summary.block_summary_root();
    assert_eq!(beacon_block_roots_tree_hash_root, block_summary_root);
    println!("Historical summary verified!");

    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Now that we have a verified set of execution block headers (and block numbers) from the beacon blocks, we can fetch the execution blocks and verify them.

    for (number, blockhash) in execution_block_number_and_hash {
        // Fetch execution blocks from the Firehose API.
        let mut eth1_client = FirehoseClient::new(Chain::Ethereum);
        let response = eth1_client.fetch_block(number).await.unwrap().unwrap();
        let eth1_block = EthBlock::try_from(response.into_inner()).unwrap();

        // Confirm that the block hash of the Ethereum block matches the hash in the block header.
        let block_header = Header::try_from(&eth1_block).unwrap();
        let eth1_block_hash = block_header.hash();
        assert_eq!(eth1_block_hash.as_slice(), &eth1_block.hash);

        // Confirm that the Ethereum block matches the Beacon block's Execution Payload.
        // This is our first major check linking the exuction layer to the consensus layer.
        assert_eq!(blockhash.into_root().as_bytes(), eth1_block_hash.as_slice());
        println!("Block number {} verified!", number);
    }

    // At this point, we have checked that the complete era's beacon blocks are correct by comparing against a historical summary from the consensus state,
    // and that the corresponding execution blocks are correct by comparing against the block headers from the verified beacon blocks.
    // Assuming that all checks passed, then the extracted data has been verified.
    println!("All checks passed!");
}
