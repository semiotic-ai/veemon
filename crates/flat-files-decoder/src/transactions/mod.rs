mod transaction_signed;
mod transaction;
mod signature;
mod access_list;

use std::u128;
use reth_primitives::{hex, TransactionSigned, U128};
use reth_primitives::proofs::{calculate_transaction_root};
use crate::protos::block::{BigInt, Block};

pub fn _check_transaction_root(block: &Block) -> anyhow::Result<()> {
    let mut transactions: Vec<TransactionSigned> = Vec::new();

    for trace in &block.transaction_traces {
        transactions.push(trace.try_into()?);
    }

    let tx_root = calculate_transaction_root(&transactions);

    if tx_root.as_bytes() != block.header.transactions_root.as_slice() {
        return Err(
            anyhow::anyhow!("Invalid transaction root, expected {}, got {}", hex::encode(block.header.transactions_root.as_slice()), hex::encode(tx_root.as_bytes()))
        );
    }

    Ok(())
}


impl TryFrom<BigInt> for u128 {
    type Error = anyhow::Error;

    fn try_from(value: BigInt) -> Result<Self, Self::Error> {
        let slice = value.bytes.as_slice();
        let n = U128::try_from_le_slice(slice)
            .ok_or(anyhow::anyhow!("BigInt too large"))?;
        Ok(u128::from_le_bytes(n.to_le_bytes()))
    }
}


#[test]
fn test_bigint_to_u128() {
    let n_u128: u128 = 12345678910;
    let n_bytes: [u8; 16] = n_u128.to_le_bytes();

    let mut bigint = BigInt::new();
    bigint.bytes = n_bytes.to_vec();

    let new_u128: u128 = bigint.try_into().unwrap();
    assert_eq!(new_u128, n_u128);
}