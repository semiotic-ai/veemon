mod transaction_signed;
mod transaction;
mod signature;
mod access_list;

use std::fs::File;
use std::str::FromStr;
use std::u128;
use protobuf::Message;
use reth_primitives::{Address, Bytes, hex, TransactionKind, TransactionSigned, TxHash, TxType, U128};
use reth_primitives::proofs::{calculate_transaction_root};
use crate::dbin::DbinFile;
use crate::protos;
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
        let n = U128::try_from_be_slice(slice)
            .ok_or(anyhow::anyhow!("BigInt too large"))?;
        Ok(u128::from_le_bytes(n.to_le_bytes()))
    }
}


#[test]
fn test_bigint_to_u128() {
    let n_u128: u128 = 12345678910;
    let n_bytes: [u8; 16] = n_u128.to_be_bytes();

    let mut bigint = BigInt::new();
    bigint.bytes = n_bytes.to_vec();

    let new_u128: u128 = bigint.try_into().unwrap();
    assert_eq!(new_u128, n_u128);
}

#[test]
fn example_file_first_tx() {
    let input_file = File::open("example0017686312.dbin").unwrap();

    let dbin_file = DbinFile::try_from(input_file).unwrap();

    let message = dbin_file.messages.first().unwrap();

    let message: protos::bstream::Block = Message::parse_from_bytes(&message).unwrap();

    let block: Block = Message::parse_from_bytes(&message.payload_buffer).unwrap();

    let trace = block.transaction_traces.first().unwrap();

    let transaction = TransactionSigned::try_from(trace).unwrap();

    let tx_details = transaction.transaction;

    assert_eq!(tx_details.value(), 0);
    assert_eq!(tx_details.nonce(), 3807);

    assert_eq!(tx_details.max_fee_per_gas(), 141_363_047_052);
    assert_eq!(tx_details.max_priority_fee_per_gas().unwrap(), 2_500_000_000);

    assert_eq!(tx_details.gas_limit(), 149_194);

    assert_eq!(tx_details.to().unwrap(), Address::from_str("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D").unwrap());

    assert_eq!(*tx_details.input(), Bytes::from_str("0x38ed1739000000000000000000000000000000000000000000000000482a1c73000800000000000000000000000000000000000000000009c14e785bf4910843948926c200000000000000000000000000000000000000000000000000000000000000a00000000000000000000000006b4b968dcecfd3d197ce04dc8925f919308153660000000000000000000000000000000000000000000000000000000064b040870000000000000000000000000000000000000000000000000000000000000002000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000370a366f402e2e41cdbbe54ecec12aae0cce1955").unwrap());

    assert_eq!(tx_details.tx_type(), TxType::EIP1559);
    assert_eq!(tx_details.chain_id(), Some(1));

    assert_eq!(*tx_details.kind(), TransactionKind::Call(Address::from_str("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D").unwrap()));

    assert_eq!(transaction.hash, TxHash::from_str("0x5d8438a6c6336b90ca42a73c4e4ea8985fdfc3e2526af38592894353fd9d0d39").unwrap())
}