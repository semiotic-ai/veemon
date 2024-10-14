use crate::transactions::error::TransactionError;
use reth_primitives::{hex, AccessList, AccessListItem, Address, B256};
use sf_protos::ethereum::r#type::v2::AccessTuple;

pub(crate) fn compute_access_list(
    access_list: &[AccessTuple],
) -> Result<AccessList, TransactionError> {
    let access_list_items: Vec<AccessListItem> = access_list
        .iter()
        .map(atuple_to_alist_item)
        .collect::<Result<Vec<AccessListItem>, TransactionError>>(
    )?;

    Ok(AccessList(access_list_items))
}

pub fn atuple_to_alist_item(tuple: &AccessTuple) -> Result<AccessListItem, TransactionError> {
    let address: Address = Address::from_slice(tuple.address.as_slice());
    let storage_keys = tuple
        .storage_keys
        .iter()
        .map(|key| {
            let key_bytes: [u8; 32] = key
                .as_slice()
                .try_into()
                .map_err(|_| TransactionError::InvalidStorageKey(hex::encode(key.clone())))?;
            Ok(B256::from(key_bytes))
        })
        .collect::<Result<Vec<B256>, TransactionError>>()?;

    Ok(AccessListItem {
        address,
        storage_keys,
    })
}
