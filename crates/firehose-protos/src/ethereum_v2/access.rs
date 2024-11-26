use crate::error::ProtosError;

use super::AccessTuple;

use alloy_eip2930::AccessListItem;
use alloy_primitives::{hex, Address, B256};

impl TryFrom<&AccessTuple> for AccessListItem {
    type Error = ProtosError;

    fn try_from(tuple: &AccessTuple) -> Result<Self, Self::Error> {
        let address = Address::from_slice(tuple.address.as_slice());

        let storage_keys = tuple
            .storage_keys
            .iter()
            .map(convert_to_b256)
            .collect::<Result<Vec<B256>, ProtosError>>()?;

        Ok(AccessListItem {
            address,
            storage_keys,
        })
    }
}

fn convert_to_b256(key: &Vec<u8>) -> Result<B256, ProtosError> {
    let key_bytes: [u8; 32] = key
        .as_slice()
        .try_into()
        .map_err(|_| ProtosError::AccessTupleStorageKeyInvalid(hex::encode(key.clone())))?;
    Ok(B256::from(key_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_fake_access_tuple() -> AccessTuple {
        AccessTuple {
            address: vec![0x11; 20],
            storage_keys: vec![vec![0xaa; 32], vec![0xbb; 32]],
        }
    }

    #[test]
    fn test_access_tuple_to_access_list_item_conversion() {
        let fake_tuple = create_fake_access_tuple();

        let access_list_item = AccessListItem::try_from(&fake_tuple).expect("Conversion failed");

        assert_eq!(access_list_item.address, Address::from([0x11; 20]));
        assert_eq!(
            access_list_item.storage_keys.len(),
            fake_tuple.storage_keys.len()
        );
    }

    #[test]
    fn test_access_tuple_with_empty_storage_keys() {
        let fake_tuple = AccessTuple {
            address: vec![0x11; 20],
            storage_keys: vec![],
        };

        let access_list_item = AccessListItem::try_from(&fake_tuple).expect("Conversion failed");

        assert_eq!(access_list_item.address, Address::from([0x11; 20]));
        assert!(access_list_item.storage_keys.is_empty());
    }

    #[test]
    fn test_access_tuple_storage_key_invalid_length() {
        let fake_tuple = AccessTuple {
            address: vec![0x11; 20],
            storage_keys: vec![vec![0xaa; 31]],
        };

        let error = AccessListItem::try_from(&fake_tuple).unwrap_err();

        assert!(matches!(
            error,
            ProtosError::AccessTupleStorageKeyInvalid(_)
        ));
    }
}
