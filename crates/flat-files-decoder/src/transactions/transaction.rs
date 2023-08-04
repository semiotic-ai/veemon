use reth_primitives::{Address, Bytes, ChainId, Transaction, TransactionKind, TxEip1559, TxEip2930, TxLegacy, TxType};
use crate::protos::block::TransactionTrace;
use crate::transactions::access_list::compute_access_list;
use crate::transactions::transaction_signed::u128_from_field;

pub const CHAIN_ID: ChainId = 1;

impl TryFrom<&TransactionTrace> for Transaction {
    type Error = anyhow::Error;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        let type_enum_val = trace.type_.enum_value()
            .map_err(|_| anyhow::anyhow!("Missing transaction type"))?;

        let tx_type = TxType::from(type_enum_val);

        let nonce = trace.nonce;
        let gas_price = u128_from_field(&trace.gas_price)?;
        let gas_limit = trace.gas_limit;

        let to = if trace.to.is_empty() {
            TransactionKind::Create
        } else {
            let address = Address::from_slice(trace.to.as_slice());
            TransactionKind::Call(address)
        };

        let chain_id = CHAIN_ID;

        let value = u128_from_field(&trace.value)?;
        let input = Bytes::from(trace.input.as_slice());

        let transaction: Transaction = match tx_type {
            TxType::Legacy => {
                let v: u8 = if trace.v.is_empty() {
                    0
                } else {
                    trace.v[0]
                };

                let chain_id: Option<ChainId> = if v == 27 || v == 28 {
                    None
                } else {
                    Some(ChainId::from(v))
                };

                Transaction::Legacy(TxLegacy {
                    chain_id,
                    nonce,
                    gas_price,
                    gas_limit,
                    to,
                    value,
                    input
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

                let max_fee_per_gas = u128_from_field(&trace.max_fee_per_gas)?;
                let max_priority_fee_per_gas = u128_from_field(&trace.max_priority_fee_per_gas)?;

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
            TxType::EIP4844 => {
                Err(anyhow::anyhow!("EIP4844 is not supported"))?
            }
        };

        Ok(transaction)
    }
}