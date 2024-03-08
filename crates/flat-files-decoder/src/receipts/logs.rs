use crate::receipts::error::ReceiptError;
use reth_primitives::{hex, Address, Bytes, Log, H256};
use std::convert::TryInto;

// type BlockLog = crate::protos::block::Log;
use sf_protos::ethereum::r#type::v2::Log as BlockLog;

pub fn map_logs(logs: &[BlockLog]) -> Result<Vec<Log>, ReceiptError> {
    logs.iter().map(block_log_to_log).collect()
}

pub fn block_log_to_log(log: &BlockLog) -> Result<Log, ReceiptError> {
    let slice: [u8; 20] = log
        .address
        .as_slice()
        .try_into()
        .map_err(|_| ReceiptError::InvalidAddress(hex::encode(log.address.clone())))?;

    let address = Address::from(slice);
    let topics = map_topics(&log.topics)?;
    let data = Bytes::from(log.data.as_slice());

    Ok(Log {
        address,
        topics,
        data,
    })
}

fn map_topics(topics: &[Vec<u8>]) -> Result<Vec<H256>, ReceiptError> {
    topics.iter().map(map_topic).collect()
}

fn map_topic(topic: &Vec<u8>) -> Result<H256, ReceiptError> {
    let slice: [u8; 32] = topic
        .as_slice()
        .try_into()
        .map_err(|_| ReceiptError::InvalidTopic(hex::encode(topic)))?;
    Ok(H256::from(slice))
}
