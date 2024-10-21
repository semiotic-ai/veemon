use primitive_types::H256;
use sf_protos::error::ProtosError;
use types::{BeaconBlock, MainnetEthSpec};

pub mod beacon_block;
pub mod beacon_state;

pub struct BlockRoot(pub H256);

impl TryFrom<sf_protos::beacon::r#type::v1::Block> for BlockRoot {
    type Error = ProtosError;

    fn try_from(beacon_block: sf_protos::beacon::r#type::v1::Block) -> Result<Self, Self::Error> {
        let lighthouse_beacon_block = BeaconBlock::<MainnetEthSpec>::try_from(beacon_block)?;
        Ok(Self(lighthouse_beacon_block.canonical_root()))
    }
}
