use crate::protos::block::AccessTuple;
use crate::transactions::error::TransactionError;
use reth_primitives::{hex, AccessList, AccessListItem, Address, H256};

pub(crate) fn compute_access_list(
    access_list: &[AccessTuple],
) -> Result<AccessList, TransactionError> {
    let access_list_items: Vec<AccessListItem> = access_list
        .iter()
        .map(AccessListItem::try_from)
        .collect::<Result<Vec<AccessListItem>, TransactionError>>(
    )?;

    Ok(AccessList(access_list_items))
}

impl TryFrom<&AccessTuple> for AccessListItem {
    type Error = TransactionError;

    fn try_from(tuple: &AccessTuple) -> Result<Self, Self::Error> {
        let address: Address = Address::from_slice(tuple.address.as_slice());
        let storage_keys = tuple
            .storage_keys
            .iter()
            .map(|key| {
                let key_bytes: [u8; 32] = key
                    .as_slice()
                    .try_into()
                    .map_err(|_| TransactionError::InvalidStorageKey(hex::encode(key.clone())))?;
                Ok(H256::from(key_bytes))
            })
            .collect::<Result<Vec<H256>, TransactionError>>()?;

        Ok(AccessListItem {
            address,
            storage_keys,
        })
    }
}
