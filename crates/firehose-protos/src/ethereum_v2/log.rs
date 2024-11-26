use alloy_primitives::{hex, Address, Bytes, B256};
use reth_primitives::LogData;

use crate::error::ProtosError;

use super::Log;

impl TryFrom<&Log> for alloy_primitives::Log {
    type Error = ProtosError;

    fn try_from(log: &Log) -> Result<Self, Self::Error> {
        let address = Address::try_from(log)?;
        let topics = log.to_topics()?;
        let log_data = Bytes::copy_from_slice(log.data.as_slice());
        let data = LogData::new_unchecked(topics, log_data);

        Ok(alloy_primitives::Log { address, data })
    }
}

const ADDRESS_SIZE: usize = 20;

impl TryFrom<&Log> for Address {
    type Error = ProtosError;

    fn try_from(log: &Log) -> Result<Self, Self::Error> {
        let slice: [u8; ADDRESS_SIZE] = log
            .address
            .as_slice()
            .try_into()
            .map_err(|_| Self::Error::LogAddressInvalid(hex::encode(log.address.clone())))?;
        Ok(Address::from(slice))
    }
}

impl Log {
    fn to_topics(&self) -> Result<Vec<B256>, ProtosError> {
        fn to_b256(slice: &[u8]) -> Result<B256, ProtosError> {
            B256::try_from(slice).map_err(|_| ProtosError::LogTopicInvalid(hex::encode(slice)))
        }

        self.topics
            .iter()
            .map(Vec::as_slice)
            .map(to_b256)
            .collect::<Result<Vec<_>, _>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_fake_log_with(address: Vec<u8>, data: Vec<u8>, topics: Vec<Vec<u8>>) -> Log {
        Log {
            address,
            data,
            topics,
            ..Default::default()
        }
    }

    fn create_fake_log() -> Log {
        create_fake_log_with(
            vec![0x11; 20],
            vec![0xde, 0xad, 0xbe, 0xef],
            vec![vec![0xff; 32], vec![0xaa; 32]],
        )
    }

    #[test]
    fn test_log_to_alloy_log_conversion() {
        let fake_log = create_fake_log();

        let alloy_log = alloy_primitives::Log::try_from(&fake_log).expect("Conversion failed");

        assert_eq!(alloy_log.address, Address::from([0x11; 20]));
        assert_eq!(alloy_log.data.data.as_ref(), fake_log.data.as_slice());
        assert_eq!(alloy_log.data.topics().len(), fake_log.topics.len());
    }

    #[test]
    fn test_log_to_address_conversion() {
        let fake_log = create_fake_log();

        let address = Address::try_from(&fake_log).expect("Conversion failed");

        assert_eq!(address, Address::from([0x11; 20]));
    }

    #[test]
    fn test_log_to_topics_conversion() {
        let fake_log = create_fake_log();

        let topics = fake_log.to_topics().expect("Conversion to topics failed");

        assert_eq!(topics.len(), 2);
        assert_eq!(topics[0], B256::from([0xff; 32]));
        assert_eq!(topics[1], B256::from([0xaa; 32]));
    }

    #[test]
    fn test_log_address_conversion_invalid_length() {
        let fake_log = create_fake_log_with(vec![0x12; 18], vec![], vec![]);

        let error = Address::try_from(&fake_log).unwrap_err();

        assert!(matches!(error, ProtosError::LogAddressInvalid(_)));
    }

    #[test]
    fn test_log_to_topics_conversion_invalid_topic_length() {
        let fake_log = create_fake_log_with(vec![], vec![], vec![vec![0xaa; 31]]);

        let error = fake_log.to_topics().unwrap_err();

        assert!(matches!(error, ProtosError::LogTopicInvalid(_)));
    }

    #[test]
    fn test_log_data_creation() {
        let fake_log = create_fake_log_with(
            vec![0x11; 20],
            vec![0xde, 0xad, 0xbe, 0xef],
            vec![vec![0xff; 32], vec![0xaa; 32]],
        );

        let alloy_log = alloy_primitives::Log::try_from(&fake_log).expect("Conversion failed");
        let log_data = alloy_log.data;

        assert_eq!(log_data.data.as_ref(), fake_log.data.as_slice());
        assert_eq!(log_data.topics().len(), fake_log.topics.len());
    }

    #[test]
    fn test_log_with_empty_data() {
        let fake_log = Log {
            data: vec![],
            ..create_fake_log()
        };

        let alloy_log = alloy_primitives::Log::try_from(&fake_log).expect("Conversion failed");

        assert_eq!(alloy_log.address, Address::from([0x11; 20]));
        assert_eq!(alloy_log.data.data.as_ref(), fake_log.data.as_slice());
        assert_eq!(alloy_log.data.topics().len(), fake_log.topics.len());
    }
}
