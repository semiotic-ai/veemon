// use crate::protos::block::{CallType, TransactionTrace};
use crate::transactions::access_list::compute_access_list;
use crate::transactions::error::TransactionError;
use crate::transactions::tx_type::map_tx_type;
use reth_primitives::{
    Address, Bytes, ChainId, Transaction, TransactionKind, TxEip1559, TxEip2930, TxLegacy, TxType,
};
use sf_protos::ethereum::r#type::v2::{BigInt, CallType, TransactionTrace};

use super::bigint_to_u128;

pub const CHAIN_ID: ChainId = 1;

pub fn trace_to_transaction(trace: &TransactionTrace) -> Result<Transaction, TransactionError> {
    let tx_type = map_tx_type(&trace.r#type)?;

    let nonce = trace.nonce;
    let trace_gas_price = match trace.gas_price.clone() {
        Some(gas_price) => gas_price,
        None => BigInt { bytes: vec![0] },
    };
    let gas_price = bigint_to_u128(trace_gas_price)?;
    let gas_limit = trace.gas_limit;

    let to = get_tx_kind(trace)?;

    let chain_id = CHAIN_ID;

    let trace_value = match trace.value.clone() {
        Some(value) => value,
        None => BigInt { bytes: vec![0] },
    };
    let value = bigint_to_u128(trace_value)?;
    let input = Bytes::from(trace.input.as_slice());

    let transaction: Transaction = match tx_type {
        TxType::Legacy => {
            let v: u8 = if trace.v.is_empty() { 0 } else { trace.v[0] };

            let chain_id: Option<ChainId> = if v == 27 || v == 28 {
                None
            } else {
                Some(CHAIN_ID)
            };

            Transaction::Legacy(TxLegacy {
                chain_id,
                nonce,
                gas_price,
                gas_limit,
                to,
                value,
                input,
            })
        }
        TxType::EIP2930 => {
            let access_list = compute_access_list(&trace.access_list)?;

            Transaction::Eip2930(TxEip2930 {
                chain_id,
                nonce,
                gas_price,
                gas_limit,
                to,
                value,
                access_list,
                input,
            })
        }
        TxType::EIP1559 => {
            let access_list = compute_access_list(&trace.access_list)?;

            let trace_max_fee_per_gas = match trace.max_fee_per_gas.clone() {
                Some(max_fee_per_gas) => max_fee_per_gas,
                None => BigInt { bytes: vec![0] },
            };
            let max_fee_per_gas = bigint_to_u128(trace_max_fee_per_gas)?;

            let trace_max_priority_fee_per_gas = match trace.max_priority_fee_per_gas.clone() {
                Some(max_priority_fee_per_gas) => max_priority_fee_per_gas,
                None => BigInt { bytes: vec![0] },
            };
            let max_priority_fee_per_gas = bigint_to_u128(trace_max_priority_fee_per_gas)?;

            Transaction::Eip1559(TxEip1559 {
                chain_id,
                nonce,
                gas_limit,
                max_fee_per_gas,
                max_priority_fee_per_gas,
                to,
                value,
                access_list,
                input,
            })
        }
    };

    Ok(transaction)
}

pub fn get_tx_kind(trace: &TransactionTrace) -> Result<TransactionKind, TransactionError> {
    let first_call = trace.calls.first().ok_or(TransactionError::MissingCall)?;

    let call_type = first_call.call_type();

    if call_type == CallType::Create {
        Ok(TransactionKind::Create)
    } else {
        let address = Address::from_slice(trace.to.as_slice());
        Ok(TransactionKind::Call(address))
    }
}
