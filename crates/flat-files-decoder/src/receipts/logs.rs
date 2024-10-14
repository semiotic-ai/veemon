use crate::receipts::error::ReceiptError;
use reth_primitives::{hex, Address, Bytes, Log, LogData, B256};
use std::convert::TryInto;

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
    let log_data = Bytes::copy_from_slice(log.data.as_slice());

    let data = LogData::new_unchecked(topics, log_data);

    Ok(Log { address, data })
}

fn map_topics(topics: &[Vec<u8>]) -> Result<Vec<B256>, ReceiptError> {
    topics.iter().map(map_topic).collect()
}

fn map_topic(topic: &Vec<u8>) -> Result<B256, ReceiptError> {
    let slice: [u8; 32] = topic
        .as_slice()
        .try_into()
        .map_err(|_| ReceiptError::InvalidTopic(hex::encode(topic)))?;
    Ok(B256::from(slice))
}
