// Copyright 2025-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use super::Block;
use firehose_rs::{Response, SingleBlockResponse};
use prost::Message;
use prost_wkt_types::Any;

use crate::error::ProtosError;

fn decode_block<M>(response: M) -> Result<Block, ProtosError>
where
    M: MessageWithBlock,
{
    let any = response
        .block()
        .ok_or(ProtosError::BlockMissingInResponse)?;
    let block = Block::decode(any.value.as_ref())?;
    Ok(block)
}
trait MessageWithBlock {
    fn block(&self) -> Option<&Any>;
}

impl MessageWithBlock for SingleBlockResponse {
    fn block(&self) -> Option<&Any> {
        self.block.as_ref()
    }
}

impl MessageWithBlock for Response {
    fn block(&self) -> Option<&Any> {
        self.block.as_ref()
    }
}

impl TryFrom<SingleBlockResponse> for Block {
    type Error = ProtosError;

    fn try_from(response: SingleBlockResponse) -> Result<Self, Self::Error> {
        decode_block(response)
    }
}

impl TryFrom<Response> for Block {
    type Error = ProtosError;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        decode_block(response)
    }
}
