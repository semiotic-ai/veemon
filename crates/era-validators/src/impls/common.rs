use primitive_types::H256;
use tree_hash::TreeHash;
use types::{
    BeaconBlock, BeaconBlockAltair, BeaconBlockBase, BeaconBlockBellatrix, BeaconBlockCapella,
    BeaconBlockDeneb, BeaconBlockElectra, EthSpec,
};

// Get the execution payload block hash from the beacon block. Depends on the beackon block type.
pub fn get_execution_payload_block_hash<E: EthSpec>(block: &BeaconBlock<E>) -> Option<H256> {
    match block {
        BeaconBlock::Base(_inner) => None,
        BeaconBlock::Altair(_inner) => None,
        BeaconBlock::Bellatrix(inner) => {
            Some(inner.body.execution_payload.execution_payload.block_hash.0)
        }
        BeaconBlock::Capella(inner) => {
            Some(inner.body.execution_payload.execution_payload.block_hash.0)
        }
        BeaconBlock::Deneb(inner) => {
            Some(inner.body.execution_payload.execution_payload.block_hash.0)
        }
        BeaconBlock::Electra(inner) => {
            Some(inner.body.execution_payload.execution_payload.block_hash.0)
        }
    }
}

pub fn compute_tree_hash_root<E: EthSpec>(block: &BeaconBlock<E>) -> H256
where
    BeaconBlockBase<E>: TreeHash,
    BeaconBlockAltair<E>: TreeHash,
    BeaconBlockBellatrix<E>: TreeHash,
    BeaconBlockCapella<E>: TreeHash,
    BeaconBlockDeneb<E>: TreeHash,
    BeaconBlockElectra<E>: TreeHash,
{
    match block {
        BeaconBlock::Base(inner) => inner.tree_hash_root(),
        BeaconBlock::Altair(inner) => inner.tree_hash_root(),
        BeaconBlock::Bellatrix(inner) => inner.tree_hash_root(),
        BeaconBlock::Capella(inner) => inner.tree_hash_root(),
        BeaconBlock::Deneb(inner) => inner.tree_hash_root(),
        BeaconBlock::Electra(inner) => inner.tree_hash_root(),
    }
}
