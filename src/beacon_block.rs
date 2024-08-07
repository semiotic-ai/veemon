use serde::{Deserialize, Serialize};
use types::{BeaconBlock, MainnetEthSpec};

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockWrapper {
    pub version: String,
    pub execution_optimistic: bool,
    pub finalized: bool,
    pub data: Data,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    pub message: BeaconBlock<MainnetEthSpec>,
}

/// Merkle proof depth for a `BeaconBlockBody` struct with 12 fields.
///
/// The proof depth is determined by finding the smallest power of 2 that is
/// greater than or equal to the number of fields. In this case, the number of
/// fields is 12, which is between 8 (2^3) and 16 (2^4).
pub const BEACON_BLOCK_BODY_PROOF_DEPTH: usize = 4;

/// The field corresponds to the index of the `eth1_data` field in the [`BeaconBlockBody`] struct:
/// <https://github.com/ethereum/annotated-spec/blob/master/deneb/beacon-chain.md#beaconblockbody>.
pub const ETH1_DATA_FIELD_INDEX: usize = 1;

#[cfg(test)]
mod tests {
    use std::cell::LazyCell;

    use super::*;

    use merkle_proof::verify_merkle_proof;
    use tree_hash::TreeHash;
    use types::{light_client_update::ETH1_DATA_INDEX, BeaconBlockBody};

    const DENEB_BLOCK_JSON: &str = include_str!("../bb-8786333.json");

    const BLOCK_WRAPPER: LazyCell<BlockWrapper> = LazyCell::new(|| {
        serde_json::from_str(DENEB_BLOCK_JSON).expect(
            "For this spike we are using a Deneb block JSON file that has been shared among contributors",
        )
    });

    #[test]
    fn test_inclusion_proof_for_block_body_given_eth1_data() {
        let block_wrapper = &BLOCK_WRAPPER;
        let block = &block_wrapper.data.message;

        let eth1_data = block.body().eth1_data();
        let eth1_data_root = eth1_data.tree_hash_root();

        let block_body = block.body_deneb().unwrap();
        let block_body_hash = block_body.tree_hash_root();

        let body = BeaconBlockBody::from(block_body.clone());
        let proof = body.block_body_merkle_proof(ETH1_DATA_INDEX).unwrap();

        let depth = BEACON_BLOCK_BODY_PROOF_DEPTH;

        assert_eq!(proof.len(), depth, "proof length should equal depth");

        assert!(verify_merkle_proof(
            eth1_data_root,
            &proof,
            depth,
            ETH1_DATA_FIELD_INDEX,
            block_body_hash
        ));
    }

    #[test]
    fn test_beacon_block_header_root_and_beacon_block_root_match() {
        let block_wrapper = &BLOCK_WRAPPER;
        let block = &block_wrapper.data.message;

        // `BeaconBlock::canonical_root` calls `tree_hash_root` on the block.
        let block_root = block.canonical_root();

        let block_header = block.block_header();
        let block_header_root = block_header.tree_hash_root();

        assert_eq!(block_root, block_header_root);
    }
}
