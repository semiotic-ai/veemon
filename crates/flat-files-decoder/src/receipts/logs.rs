use reth_primitives::{Address, Bytes, H256, hex, Log};
use std::convert::TryInto;
use crate::receipts::error::InvalidReceiptError;

type BlockLog = crate::protos::block::Log;

pub(crate) fn map_logs(logs: &[BlockLog]) -> Result<Vec<Log>, InvalidReceiptError> {
    logs.iter().map(Log::try_from).collect()
}

impl TryFrom<&BlockLog> for Log {
    type Error = InvalidReceiptError;

    fn try_from(log: &BlockLog) -> Result<Self, Self::Error> {
        let slice: [u8;20] = log.address.as_slice().try_into()
            .map_err(|_| InvalidReceiptError::Address(hex::encode(log.address.clone())))?;

        let address = Address::from(slice);
        let topics = map_topics(&log.topics)?;
        let data = Bytes::from(log.data.as_slice());

        Ok(Self {
            address,
            topics,
            data
        })
    }
}

fn map_topics(topics: &[Vec<u8>]) -> Result<Vec<H256>, InvalidReceiptError> {
    topics.iter().map(map_topic).collect()
}

fn map_topic(topic: &Vec<u8>) -> Result<H256, InvalidReceiptError> {
    let slice: [u8;32] = topic.as_slice().try_into()
        .map_err(|_| InvalidReceiptError::Topic(hex::encode(topic)))?;
    Ok(H256::from(slice))
}