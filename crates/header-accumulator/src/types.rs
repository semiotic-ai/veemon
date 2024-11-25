// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use alloy_primitives::{Uint, B256};
use ethportal_api::{types::execution::accumulator::HeaderRecord, Header};
use firehose_protos::{BlockHeader, EthBlock as Block};

use crate::errors::EraValidateError;

/// Extension of header with conversion traits
///
/// It's capable of converting to HeaderRecord to be used inside Epochs.
/// It can also convert to firehose protos.
/// You can extract the full header as an option
#[derive(Clone)]
pub struct ExtHeaderRecord {
    pub block_hash: B256,
    pub total_difficulty: Uint<256, 4>,
    pub block_number: u64,
    pub full_header: Option<Header>,
}

impl From<ExtHeaderRecord> for HeaderRecord {
    fn from(
        ExtHeaderRecord {
            block_hash,
            total_difficulty,
            ..
        }: ExtHeaderRecord,
    ) -> Self {
        HeaderRecord {
            block_hash,
            total_difficulty,
        }
    }
}

impl TryFrom<ExtHeaderRecord> for Header {
    type Error = EraValidateError;

    fn try_from(ext: ExtHeaderRecord) -> Result<Self, Self::Error> {
        ext.full_header
            .ok_or(EraValidateError::ExtHeaderRecordError(ext.block_number))
    }
}

impl From<&ExtHeaderRecord> for HeaderRecord {
    fn from(ext: &ExtHeaderRecord) -> Self {
        HeaderRecord {
            block_hash: ext.block_hash,
            total_difficulty: ext.total_difficulty,
        }
    }
}

/// Decodes a [`ExtHeaderRecord`] from a [`Block`]. A [`BlockHeader`] must be present in the block,
/// otherwise validating headers won't be possible
impl TryFrom<&Block> for ExtHeaderRecord {
    type Error = EraValidateError;

    fn try_from(block: &Block) -> Result<Self, Self::Error> {
        let header: &BlockHeader = block
            .header
            .as_ref()
            .ok_or(EraValidateError::HeaderDecodeError)?;

        let total_difficulty = header
            .total_difficulty
            .as_ref()
            .ok_or(EraValidateError::HeaderDecodeError)?;

        Ok(ExtHeaderRecord {
            block_number: block.number,
            block_hash: B256::from_slice(&block.hash),
            total_difficulty: Uint::from_be_slice(total_difficulty.bytes.as_slice()),
            full_header: Some(block.try_into()?),
        })
    }
}
