use reth_primitives::{AccessList, AccessListItem, Address, H256};
use crate::protos::block::AccessTuple;

pub(crate) fn compute_access_list(access_list: &[AccessTuple]) -> anyhow::Result<AccessList> {
    let access_list_items: Vec<AccessListItem> = access_list.iter()
        .map(AccessListItem::try_from).collect::<anyhow::Result<Vec<AccessListItem>>>()?;

    Ok(AccessList(access_list_items))
}

impl TryFrom<&AccessTuple> for AccessListItem {
    type Error = anyhow::Error;

    fn try_from(tuple: &AccessTuple) -> Result<Self, Self::Error> {
        let address: Address = Address::from_slice(tuple.address.as_slice());
        let storage_keys = tuple.storage_keys.iter().map(|key| {
            let key_bytes: [u8;32] = key.as_slice().try_into()?;
            Ok(H256::from(key_bytes))
        }).collect::<anyhow::Result<Vec<H256>>>()?;

        Ok(AccessListItem {
            address,
            storage_keys
        })
    }
}