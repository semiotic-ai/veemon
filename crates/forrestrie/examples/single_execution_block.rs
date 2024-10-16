use ethportal_api::Header;
use firehose_client::{Chain, FirehoseClient};
use forrestrie::{
    beacon_block::{
        HistoricalDataProofs, BEACON_BLOCK_BODY_PROOF_DEPTH, EXECUTION_PAYLOAD_FIELD_INDEX,
    },
    beacon_state::{compute_block_roots_proof_only, HeadState, HISTORY_TREE_DEPTH},
    BlockRoot,
};
use futures::StreamExt;
use merkle_proof::verify_merkle_proof;
use sf_protos::{beacon, ethereum};
use tree_hash::TreeHash;
use types::{
    light_client_update::EXECUTION_PAYLOAD_INDEX, BeaconBlock, BeaconBlockBody,
    BeaconBlockBodyDeneb, EthSpec, ExecPayload, Hash256, MainnetEthSpec, Vector,
};
/// This block relates to the slot represented by [`BEACON_SLOT_NUMBER`].
/// The execution block is in the execution payload of the Beacon block in slot [`BEACON_SLOT_NUMBER`].
const EXECUTION_BLOCK_NUMBER: u64 = 20759937;
/// This slot is the slot of the Beacon block that contains the execution block with [`EXECUTION_BLOCK_NUMBER`].
const BEACON_SLOT_NUMBER: u64 = 9968872; // <- this is the one that pairs with 20759937
#[tokio::main]
async fn main() {
    let handle: tokio::task::JoinHandle<HeadState<MainnetEthSpec>> = tokio::spawn(async {
        serde_json::from_str(std::include_str!("../../../state-9969663.json")).unwrap()
    });
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Get the Ethereum block.
    let mut eth1_client = FirehoseClient::new(Chain::Ethereum);
    let response = eth1_client
        .fetch_block(EXECUTION_BLOCK_NUMBER)
        .await
        .unwrap()
        .unwrap();
    let eth1_block = ethereum::r#type::v2::Block::try_from(response.into_inner()).unwrap();
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // And get the Beacon block.
    let mut beacon_client = FirehoseClient::new(Chain::Beacon);
    let response = beacon_client
        .fetch_block(BEACON_SLOT_NUMBER)
        .await
        .unwrap()
        .unwrap();
    let beacon_block = beacon::r#type::v1::Block::try_from(response.into_inner()).unwrap();
    assert_eq!(beacon_block.slot, BEACON_SLOT_NUMBER);
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Confirm that the block hash of the Ethereum block matches the hash in the block header.
    let block_header = Header::try_from(&eth1_block).unwrap();
    let eth1_block_hash = block_header.hash();
    assert_eq!(eth1_block_hash.as_slice(), &eth1_block.hash);
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Convert the Beacon block to a Lighthouse BeaconBlock.
    let lighthouse_beacon_block = BeaconBlock::<MainnetEthSpec>::try_from(beacon_block.clone())
        .expect("Failed to convert Beacon block to Lighthouse BeaconBlock");
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Check the root of the Beacon block.
    let lighthouse_beacon_block_root = lighthouse_beacon_block.canonical_root();
    assert_eq!(
        lighthouse_beacon_block_root.as_bytes(),
        beacon_block.root.as_slice()
    );
    let Some(beacon::r#type::v1::block::Body::Deneb(body)) = beacon_block.body else {
        panic!("Unsupported block version!");
    };
    let block_body: BeaconBlockBodyDeneb<MainnetEthSpec> = body.try_into().unwrap();
    // Confirm that the Beacon block's Execution Payload matches the Ethereum block we fetched.
    assert_eq!(
        block_body.execution_payload.block_number(),
        EXECUTION_BLOCK_NUMBER
    );
    // Confirm that the Ethereum block matches the Beacon block's Execution Payload.
    assert_eq!(
        block_body
            .execution_payload
            .block_hash()
            .into_root()
            .as_bytes(),
        eth1_block_hash.as_slice()
    );
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Confirm that the Execution Payload is included in the Beacon block.
    let block_body_hash = block_body.tree_hash_root();
    let execution_payload = &block_body.execution_payload;
    let execution_payload_root = execution_payload.tree_hash_root();
    let body = BeaconBlockBody::from(block_body.clone());
    let proof = body.compute_merkle_proof(EXECUTION_PAYLOAD_INDEX).unwrap();
    let depth = BEACON_BLOCK_BODY_PROOF_DEPTH;
    assert!(verify_merkle_proof(
        execution_payload_root,
        &proof,
        depth,
        EXECUTION_PAYLOAD_FIELD_INDEX,
        block_body_hash
    ));
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // This is the head state on file we started reading at the start of this script.
    let head_state = handle.await.unwrap();
    // The index of the block in the complete era of block roots.
    // Beacon chain slot numbers are zero-based; genesis slot is 0.
    // We need this to calculate the merkle inclusion proof later.
    // If this is the first/last block of the era, the index is 0/8191.
    let index = lighthouse_beacon_block.slot().as_usize() % 8192;
    let block_root_at_index = head_state
        .data()
        .block_roots()
        .get(index)
        .expect("Block root not found at index");
    assert_eq!(block_root_at_index, &lighthouse_beacon_block_root);
    println!("Requesting 8192 blocks for the era... (this takes a while)");
    let mut beacon_stream_client = FirehoseClient::new(Chain::Beacon);
    let stream = beacon_stream_client
        .stream_beacon_with_retry((BEACON_SLOT_NUMBER / 8192) * 8192, 8192)
        .await;
    tokio::pin!(stream);
    let mut block_roots: Vec<Hash256> = Vec::with_capacity(8192);
    while let Some(block) = stream.next().await {
        let root = BlockRoot::try_from(block).unwrap();
        block_roots.push(root.0);
    }
    assert_eq!(block_roots.len(), 8192);
    // Compute the proof of the block's inclusion in the block roots.
    let proof = compute_block_roots_proof_only::<MainnetEthSpec>(&block_roots, index).unwrap();
    let block_roots_tree_hash_root =
        Vector::<Hash256, <MainnetEthSpec as EthSpec>::SlotsPerHistoricalRoot>::new(block_roots)
            .unwrap()
            .tree_hash_root();
    assert_eq!(proof.len(), HISTORY_TREE_DEPTH);
    // Verify the proof.
    assert!(
        verify_merkle_proof(
            lighthouse_beacon_block_root, // the root of the block
            &proof,                       // the proof of the block's inclusion in the block roots
            HISTORY_TREE_DEPTH,           // the depth of the block roots tree
            index,                        // the index of the block in the era
            block_roots_tree_hash_root    // The root of the block roots
        ),
        "Merkle proof verification failed"
    );
    println!("All checks passed!");
}
