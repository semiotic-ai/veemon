use firehose_protos::error::ProtosError;
use primitive_types::H256;
use types::{BeaconBlock, MainnetEthSpec};

pub mod beacon_block;
pub mod beacon_state;

pub struct BlockRoot(pub H256);

impl TryFrom<firehose_protos::beacon_v1::Block> for BlockRoot {
    type Error = ProtosError;

    fn try_from(beacon_block: firehose_protos::beacon_v1::Block) -> Result<Self, Self::Error> {
        let lighthouse_beacon_block = BeaconBlock::<MainnetEthSpec>::try_from(beacon_block)?;
        Ok(Self(lighthouse_beacon_block.canonical_root()))
    }
}
