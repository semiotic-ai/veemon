use sf_protos::firehose::v2::{
    single_block_request::{BlockNumber, Reference},
    Request, SingleBlockRequest,
};

pub fn create_block_request(num: u64) -> SingleBlockRequest {
    SingleBlockRequest {
        reference: Some(Reference::BlockNumber(BlockNumber { num })),
        ..Default::default()
    }
}

pub enum BlocksRequested {
    All,
    FinalOnly,
}

pub fn create_blocks_request(
    start_block_num: i64,
    stop_block_num: u64,
    blocks_requested: BlocksRequested,
) -> Request {
    use BlocksRequested::*;
    Request {
        start_block_num,
        stop_block_num,
        final_blocks_only: match blocks_requested {
            All => false,
            FinalOnly => true,
        },
        ..Default::default()
    }
}
