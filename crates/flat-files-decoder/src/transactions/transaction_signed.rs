use protobuf::MessageField;
use reth_primitives::{Address, Bytes, ChainId, Signature, Transaction, TransactionKind, TransactionSigned, TxEip1559, TxEip2930, TxHash, TxLegacy, TxType, U256};
use crate::protos::block::{BigInt, TransactionTrace};
use crate::transactions::compute_access_list;

const CHAIN_ID: ChainId = 1;

impl TryFrom<&TransactionTrace> for TransactionSigned {
    type Error = anyhow::Error;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        let hash_bytes: [u8; 32] = trace.hash.as_slice().try_into()?;
        let hash = TxHash::from(hash_bytes);
        let type_enum_val = trace.type_.enum_value().map_err(|_| anyhow::anyhow!("Invalid transaction type"))?;
        let tx_type = TxType::from(type_enum_val);

        let nonce = trace.nonce;
        let gas_price: u128 = u128_from_field(&trace.gas_price)?;
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
                Transaction::Legacy(TxLegacy {
                    chain_id: Some(chain_id),
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

        let r_bytes: [u8;32] = trace.r.as_slice().try_into()?;
        let r = U256::from_le_bytes(r_bytes);

        let s_bytes: [u8;32] = trace.s.as_slice().try_into()?;
        let s = U256::from_le_bytes(s_bytes);

        let mut odd_y_parity: bool = false;
        let mut v: u64 = 37;
        if trace.v.is_empty() {
            odd_y_parity = false;
        } else {
            v = trace.v[0] as u64;
            if v == 1 {
                v = 36 + 2 * CHAIN_ID;
            } else if v == 27 || v == 28 {
                v += 8 + 2 * CHAIN_ID;
            }
            odd_y_parity = v - 2 * CHAIN_ID - 35 == 1;
        }

        let signature = Signature {
            r,
            s,
            odd_y_parity,
        };

        let calc_v = signature.v(Some(1));

        if calc_v != v {
            println!("expected: {}, got: {}", v, calc_v);
            return Err(anyhow::anyhow!("Invalid v value"));
        }

        Ok(TransactionSigned {
            hash,
            signature,
            transaction,
        })
    }
}

fn u128_from_field(field: &MessageField<BigInt>) -> anyhow::Result<u128> {
    Ok(field.get_or_default().clone().try_into()?)
}