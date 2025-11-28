// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use primitive_types::H256;
use tree_hash::TreeHash;
use types::{
    BeaconBlock, BeaconBlockAltair, BeaconBlockBase, BeaconBlockBellatrix, BeaconBlockCapella,
    BeaconBlockDeneb, BeaconBlockElectra, EthSpec,
};

/// get the execution payload block hash from the beacon block. depends on the beacon block type.
pub fn get_execution_payload_block_hash<E: EthSpec>(block: &BeaconBlock<E>) -> Option<H256> {
    match block {
        BeaconBlock::Base(_inner) => None,
        BeaconBlock::Altair(_inner) => None,
        BeaconBlock::Bellatrix(inner) => Some(
            inner
                .body
                .execution_payload
                .execution_payload
                .block_hash
                .0
                 .0
                .into(),
        ),
        BeaconBlock::Capella(inner) => Some(
            inner
                .body
                .execution_payload
                .execution_payload
                .block_hash
                .0
                 .0
                .into(),
        ),
        BeaconBlock::Deneb(inner) => Some(
            inner
                .body
                .execution_payload
                .execution_payload
                .block_hash
                .0
                 .0
                .into(),
        ),
        BeaconBlock::Electra(inner) => Some(
            inner
                .body
                .execution_payload
                .execution_payload
                .block_hash
                .0
                 .0
                .into(),
        ),
        BeaconBlock::Fulu(inner) => Some(
            inner
                .body
                .execution_payload
                .execution_payload
                .block_hash
                .0
                 .0
                .into(),
        ),
        BeaconBlock::Gloas(inner) => Some(
            inner
                .body
                .execution_payload
                .execution_payload
                .block_hash
                .0
                 .0
                .into(),
        ),
    }
}

/// compute the tree hash root for a beacon block
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
        BeaconBlock::Base(inner) => inner.tree_hash_root().0.into(),
        BeaconBlock::Altair(inner) => inner.tree_hash_root().0.into(),
        BeaconBlock::Bellatrix(inner) => inner.tree_hash_root().0.into(),
        BeaconBlock::Capella(inner) => inner.tree_hash_root().0.into(),
        BeaconBlock::Deneb(inner) => inner.tree_hash_root().0.into(),
        BeaconBlock::Electra(inner) => inner.tree_hash_root().0.into(),
        BeaconBlock::Fulu(inner) => inner.tree_hash_root().0.into(),
        BeaconBlock::Gloas(inner) => inner.tree_hash_root().0.into(),
    }
}
