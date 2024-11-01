use flat_files_decoder::dbin::DbinFile;

use alloy_primitives::{Parity, TxHash, TxKind, U256};
use firehose_protos::{
    bstream::v1::Block as BstreamBlock,
    ethereum_v2::{transaction_trace::Type, Block},
};
use prost::Message;
use reth_primitives::{TransactionSigned, TxType};
use std::{fs::File, io::BufReader, str::FromStr};

const TEST_ASSET_PATH: &str = "../../test-assets";

#[test]
fn legacy_tx() {
    let mut input_file =
        BufReader::new(File::open(format!("{TEST_ASSET_PATH}/example0017686312.dbin")).unwrap());

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
    let mut input_file = BufReader::new(
        File::open(format!("{TEST_ASSET_PATH}/example-create-17686085.dbin")).unwrap(),
    );

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
