use crate::BlockNumber;

use super::{single_block_request::Reference, SingleBlockRequest};

impl SingleBlockRequest {
    /// Create a Firehose [`SingleBlockRequest`] for the given *block number*.
    pub fn new(num: u64) -> SingleBlockRequest {
        SingleBlockRequest {
            reference: Some(Reference::BlockNumber(BlockNumber { num })),
            ..Default::default()
        }
    }
}
