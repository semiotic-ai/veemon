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
