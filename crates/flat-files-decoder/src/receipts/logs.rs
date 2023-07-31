use reth_primitives::{Address, Bytes, H256, Log};
use std::convert::TryInto;

pub(crate) fn map_logs(logs: &Vec<crate::protos::block::Log>) -> anyhow::Result<Vec<Log>> {
    logs.iter().map(map_log).collect()
}

fn map_log(log: &crate::protos::block::Log) -> anyhow::Result<Log> {
    let slice: [u8;20] = log.address.as_slice().try_into()?;

    let address = Address::from(slice);
    let topics = map_topics(&log.topics)?;
    let data = Bytes::from(log.data.as_slice());

    Ok(Log {
        address,
        topics,
        data
    })
}

fn map_topics(topics: &Vec<Vec<u8>>) -> anyhow::Result<Vec<H256>> {
    topics.iter().map(|topic| {
        let slice: [u8;32] = topic.as_slice().try_into()?;
        Ok(H256::from(slice))
    }).collect()
}