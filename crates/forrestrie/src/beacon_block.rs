use serde::{Deserialize, Serialize};
use tree_hash::TreeHash;
use types::{
    beacon_block_body::NUM_BEACON_BLOCK_BODY_HASH_TREE_ROOT_LEAVES, light_client_update,
    BeaconBlock, BeaconBlockBody, Error, EthSpec, ForkName, Hash256, MainnetEthSpec,
};

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

// Eth1Data is a [`BeaconBlockBody`] top-level field, subtract off the generalized indices
// for the internal nodes. Result should be 1, the field offset of the execution
// payload in the `BeaconBlockBody`:
// https://github.com/ethereum/consensus-specs/blob/dev/specs/deneb/beacon-chain.md#beaconblockbody
pub const ETH1_DATA_INDEX: usize = 17;

/// The field corresponds to the index of the `eth1_data` field in the [`BeaconBlockBody`] struct:
/// <https://github.com/ethereum/annotated-spec/blob/master/deneb/beacon-chain.md#beaconblockbody>.
pub const ETH1_DATA_FIELD_INDEX: usize = 1;

// ExecutionPayload is a [`BeaconBlockBody`] top-level field, subtract off the generalized indices
// for the internal nodes. Result should be 9, the field offset of the execution
// payload in the `BeaconBlockBody`:
// https://github.com/ethereum/consensus-specs/blob/dev/specs/deneb/beacon-chain.md#beaconblockbody
pub const EXECUTION_PAYLOAD_INDEX: usize = 25;

/// The field corresponds to the index of the `execution_payload` field in the [`BeaconBlockBody`] struct:
/// <https://github.com/ethereum/annotated-spec/blob/master/deneb/beacon-chain.md#beaconblockbody>.
pub const EXECUTION_PAYLOAD_FIELD_INDEX: usize = 9;

pub trait HistoricalDataProofs {
    fn compute_merkle_proof(&self, index: usize) -> Result<Vec<Hash256>, Error>;
}

impl<E: EthSpec> HistoricalDataProofs for BeaconBlockBody<E> {
    fn compute_merkle_proof(&self, index: usize) -> Result<Vec<Hash256>, Error> {
        let field_index = match index {
            ETH1_DATA_INDEX => index
                .checked_sub(NUM_BEACON_BLOCK_BODY_HASH_TREE_ROOT_LEAVES)
                .ok_or(Error::IndexNotSupported(index))?,
            EXECUTION_PAYLOAD_INDEX => index
                .checked_sub(NUM_BEACON_BLOCK_BODY_HASH_TREE_ROOT_LEAVES)
                .ok_or(Error::IndexNotSupported(index))?,
            _ => return Err(Error::IndexNotSupported(index)),
        };

        let attestations_root = if self.fork_name() > ForkName::Electra {
            self.attestations_electra()?.tree_hash_root()
        } else {
            self.attestations_base()?.tree_hash_root()
        };

        let attester_slashings_root = if self.fork_name() > ForkName::Electra {
            self.attester_slashings_electra()?.tree_hash_root()
        } else {
            self.attester_slashings_base()?.tree_hash_root()
        };

        let mut leaves = vec![
            self.randao_reveal().tree_hash_root(),
            self.eth1_data().tree_hash_root(),
            self.graffiti().tree_hash_root(),
            self.proposer_slashings().tree_hash_root(),
            attester_slashings_root,
            attestations_root,
            self.deposits().tree_hash_root(),
            self.voluntary_exits().tree_hash_root(),
        ];

        if let Ok(sync_aggregate) = self.sync_aggregate() {
            leaves.push(sync_aggregate.tree_hash_root())
        }

        if let Ok(execution_payload) = self.execution_payload() {
            leaves.push(execution_payload.tree_hash_root())
        }

        if let Ok(bls_to_execution_changes) = self.bls_to_execution_changes() {
            leaves.push(bls_to_execution_changes.tree_hash_root())
        }

        if let Ok(blob_kzg_commitments) = self.blob_kzg_commitments() {
            leaves.push(blob_kzg_commitments.tree_hash_root())
        }

        let depth = light_client_update::EXECUTION_PAYLOAD_PROOF_LEN;
        let tree = merkle_proof::MerkleTree::create(&leaves, depth);
        let (_, proof) = tree.generate_proof(field_index, depth)?;

        Ok(proof)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::LazyCell;

    use super::*;

    use ethportal_api::Header;
    use firehose_client::{
        client::{channel::fetch_client, endpoint::Firehose},
        request::{create_request, FirehoseRequest},
    };
    use merkle_proof::verify_merkle_proof;
    use sf_protos::ethereum::r#type::v2::Block;
    use types::ExecPayload;

    /// Deneb block JSON file shared among contributors.
    /// The block hash is `0x5dde05ab1da7f768ed3ea2d53c6fa0d79c0c2283e52bb0d00842a4bdbf14c0ab`.
    const DENEB_BLOCK_JSON: &str = include_str!("../../../bb-8786333.json");

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
        let proof = body.compute_merkle_proof(ETH1_DATA_INDEX).unwrap();

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

    /// Demonstrate that we can verify the inclusion proof for the execution payload field in the block body.
    /// The execution payload block hash should match the block hash of the execution block.
    #[test]
    fn test_inclusion_proof_for_block_body_given_execution_payload() {
        let block_wrapper = &BLOCK_WRAPPER;
        let block = &block_wrapper.data.message;

        let execution_payload = block.body().execution_payload().unwrap();
        let execution_payload_root = execution_payload.tree_hash_root();

        let block_body = block.body_deneb().unwrap();
        let block_body_hash = block_body.tree_hash_root();

        let body = BeaconBlockBody::from(block_body.clone());
        let proof = body.compute_merkle_proof(EXECUTION_PAYLOAD_INDEX).unwrap();

        let depth = BEACON_BLOCK_BODY_PROOF_DEPTH;

        assert_eq!(proof.len(), depth, "proof length should equal depth");

        assert!(verify_merkle_proof(
            execution_payload_root,
            &proof,
            depth,
            EXECUTION_PAYLOAD_FIELD_INDEX,
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

    #[tokio::test]
    async fn validate_single_execution_block_proof() {
        const BLOCK_NUMBER: u64 = 19_584_570;

        let mut eth1_client = fetch_client(Firehose::Ethereum).await.unwrap();

        let mut request = create_request(BLOCK_NUMBER);

        request.insert_api_key_if_provided(Firehose::Ethereum);

        let response = eth1_client.block(request).await.unwrap();

        let block = Block::try_from(response.into_inner()).unwrap();

        let block_header = Header::try_from(&block).unwrap();
        let block_hash = block_header.hash();

        assert_eq!(block_hash.as_slice(), &block.hash);

        let consensus_block = &BLOCK_WRAPPER.data.message;
        let block_body = consensus_block.body_deneb().unwrap();
        let consensus_block_body = BeaconBlockBody::from(block_body.clone());
        let execution_payload = consensus_block_body
            .execution_payload_deneb()
            .expect("Failed to get execution payload");
        let execution_block_hash = execution_payload.block_hash();

        let block_num = execution_payload.block_number();
        assert_eq!(block_num, BLOCK_NUMBER);

        assert_eq!(
            block_hash.as_slice(),
            execution_block_hash.into_root().as_bytes()
        );
    }
}
