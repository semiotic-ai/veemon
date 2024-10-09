use serde::{Deserialize, Serialize};
use tree_hash::TreeHash;
use types::{
    beacon_block_body::NUM_BEACON_BLOCK_BODY_HASH_TREE_ROOT_LEAVES,
    light_client_update::{self, EXECUTION_PAYLOAD_INDEX},
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

/// The field corresponds to the index of the `execution_payload` field in the [`BeaconBlockBody`] struct:
/// <https://github.com/ethereum/annotated-spec/blob/master/deneb/beacon-chain.md#beaconblockbody>.
pub const EXECUTION_PAYLOAD_FIELD_INDEX: usize = 9;

pub trait HistoricalDataProofs {
    fn compute_merkle_proof(&self, index: usize) -> Result<Vec<Hash256>, Error>;
}

impl<E: EthSpec> HistoricalDataProofs for BeaconBlockBody<E> {
    fn compute_merkle_proof(&self, index: usize) -> Result<Vec<Hash256>, Error> {
        let field_index = match index {
            index if index == EXECUTION_PAYLOAD_INDEX => index
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

    use merkle_proof::verify_merkle_proof;

    /// Deneb block JSON file shared among contributors.
    /// The execution payload block hash is `0xad1b9aa0d3315a08d38e258a914721630aa5d32efef8d02607555fe8ae92d7fc`.
    const DENEB_BLOCK_JSON: &str = include_str!("../../../bb-8786333.json");

    const BLOCK_WRAPPER: LazyCell<BlockWrapper> = LazyCell::new(|| {
        serde_json::from_str(DENEB_BLOCK_JSON).expect(
            "For this spike we are using a Deneb block JSON file that has been shared among contributors",
        )
    });

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

        insta::assert_debug_snapshot!(block.slot(), @
            "Slot(8786333)");

        // `BeaconBlock::canonical_root` calls `tree_hash_root` on the block.
        let block_root = block.canonical_root();

        // See, for example, https://beaconcha.in/slot/8786333 and https://beaconscan.com/slot/8786333
        insta::assert_debug_snapshot!(block_root, @"0x063d4cf1a4f85d228d9eae17a9ab7df8b13de51e7a1988342a901575cce79613");

        let block_header = block.block_header();
        let block_header_root = block_header.tree_hash_root();

        assert_eq!(block_root, block_header_root);

        // This is to show that block hash and block body hash are different.
        let body = block.body_deneb().unwrap();
        let body_hash = body.tree_hash_root();
        insta::assert_debug_snapshot!(body_hash, @"0xc15e821344ce5b201e2938248921743da8a07782168456929c8cef9f25a4cb02");
    }
}
