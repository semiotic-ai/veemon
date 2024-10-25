use crate::transactions::error::TransactionError;
use alloy_eip2930::{AccessList, AccessListItem};
use firehose_protos::ethereum_v2::AccessTuple;

pub(crate) fn compute_access_list(
    access_list: &[AccessTuple],
) -> Result<AccessList, TransactionError> {
    let access_list_items = access_list
        .iter()
        .map(AccessListItem::try_from)
        .collect::<Result<Vec<AccessListItem>, _>>()?;

    Ok(AccessList(access_list_items))
}
