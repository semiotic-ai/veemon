use flat_files_decoder::dbin::DbinFile;

use alloy_primitives::{Address, Bytes, Parity, TxHash, TxKind, U256};
use firehose_protos::ethereum_v2::Block;
use firehose_protos::{bstream::v1::Block as BstreamBlock, ethereum_v2::transaction_trace::Type};
use prost::Message;
use reth_primitives::TransactionSigned;
use reth_primitives::TxType;
use std::{fs::File, io::BufReader, str::FromStr};

#[test]
fn example_file_first_tx() {
    let mut input_file = BufReader::new(File::open("example0017686312.dbin").unwrap());

    let dbin_file = DbinFile::try_from_read(&mut input_file).unwrap();

    let message = dbin_file.messages.first().unwrap();

    let message = BstreamBlock::decode(message.as_slice()).unwrap();

    let block = Block::decode(message.payload_buffer.as_slice()).unwrap();

    let trace = block.transaction_traces.first().unwrap();

    let transaction = TransactionSigned::try_from(trace).unwrap();

    let tx_details = transaction.transaction;

    assert_eq!(tx_details.value(), U256::from(0));
    assert_eq!(tx_details.nonce(), 3807);

    assert_eq!(tx_details.max_fee_per_gas(), 141_363_047_052);
    assert_eq!(
        tx_details.max_priority_fee_per_gas().unwrap(),
        2_500_000_000
    );

    assert_eq!(tx_details.gas_limit(), 149_194);

    assert_eq!(
        tx_details.to().unwrap(),
        Address::from_str("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D").unwrap()
    );

    assert_eq!(*tx_details.input(), Bytes::from_str("0x38ed1739000000000000000000000000000000000000000000000000482a1c73000800000000000000000000000000000000000000000009c14e785bf4910843948926c200000000000000000000000000000000000000000000000000000000000000a00000000000000000000000006b4b968dcecfd3d197ce04dc8925f919308153660000000000000000000000000000000000000000000000000000000064b040870000000000000000000000000000000000000000000000000000000000000002000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000370a366f402e2e41cdbbe54ecec12aae0cce1955").unwrap());

    assert_eq!(tx_details.tx_type(), TxType::Eip1559);
    assert_eq!(tx_details.chain_id(), Some(1));

    assert_eq!(
        tx_details.kind(),
        TxKind::Call(Address::from_str("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D").unwrap())
    );

    let signature = transaction.signature;

    assert_eq!(signature.v(), Parity::Parity(false));
    assert_eq!(
        signature.r(),
        U256::from_str("0x0c8ee5280894c443ad128321d3f682c257afef878c5be9c18028b9570414213e")
            .unwrap()
    );
    assert_eq!(
        signature.s(),
        U256::from_str("0x0318b26186566acbe046e9d9caaa02444f730f4e9023c835530e622e357f3fdd")
            .unwrap()
    );

    assert_eq!(
        transaction.hash,
        TxHash::from_str("0x5d8438a6c6336b90ca42a73c4e4ea8985fdfc3e2526af38592894353fd9d0d39")
            .unwrap()
    )
}

#[test]
fn legacy_tx() {
    let mut input_file = BufReader::new(File::open("example0017686312.dbin").unwrap());

    let dbin_file = DbinFile::try_from_read(&mut input_file).unwrap();

    let message = dbin_file.messages.first().unwrap();

    let message = BstreamBlock::decode(message.as_slice()).unwrap();

    let block = Block::decode(message.payload_buffer.as_slice()).unwrap();

    let trace = block
        .transaction_traces
        .iter()
        .find(|t| Type::try_from(t.r#type).unwrap() == Type::TrxTypeLegacy)
        .unwrap();

    let transaction = TransactionSigned::try_from(trace).unwrap();

    let signature = transaction.signature;

    assert_eq!(transaction.transaction.tx_type(), TxType::Legacy);

    assert_eq!(transaction.transaction.chain_id(), Some(1));

    assert_eq!(signature.v(), Parity::Parity(true));
    assert_eq!(
        signature.r(),
        U256::from_str("0x44c2b52e2e291f1c13f572ff786039d4520955b640eae90d3c3d9a2117b0638b")
            .unwrap()
    );
    assert_eq!(
        signature.s(),
        U256::from_str("0x2a15dc9fd6c495a4a65015c3c6e41f480626741e78008091415b26410e209902")
            .unwrap()
    );

    assert_eq!(
        transaction.hash,
        TxHash::from_str("0xa074bc87b8bb4120b77c5991f9d9fe2e1df45c58d891aa1aafb0edd5bf914f8f")
            .unwrap()
    );
}

#[test]
fn create_tx() {
    let mut input_file = BufReader::new(File::open("example-create-17686085.dbin").unwrap());

    let dbin_file = DbinFile::try_from_read(&mut input_file).unwrap();

    let message = dbin_file.messages.first().unwrap();

    let message = BstreamBlock::decode(message.as_slice()).unwrap();

    let block = Block::decode(message.payload_buffer.as_slice()).unwrap();

    let trace = block
        .transaction_traces
        .iter()
        .find(|t| t.index == 141)
        .unwrap();

    let transaction = TransactionSigned::try_from(trace).unwrap();

    let tx_details = transaction.transaction;

    assert_eq!(tx_details.kind(), TxKind::Create);
    assert_eq!(transaction.hash.as_slice(), trace.hash.as_slice());
}