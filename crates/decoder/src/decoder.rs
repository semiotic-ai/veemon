// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::File,
    io::{BufReader, Cursor, Read},
};

use crate::{dbin::read_block_from_reader, error::DecoderError, DbinFile};
use firehose_protos::{
    BigInt, BlockHeader, BstreamBlock, EthBlock as Block, Timestamp, Uint64NestedArray,
};
use parquet::{
    file::reader::{FileReader, SerializedFileReader},
    record::RowAccessor,
};
use prost::Message;
use tracing::{error, info};

/// Work with data compression, including zstd.
#[derive(Clone, Copy, Debug, Default)]
pub enum Compression {
    /// Zstd compression.
    Zstd,
    /// No compression.
    #[default]
    None,
}

impl From<&str> for Compression {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "true" | "1" => Compression::Zstd,
            _ => Compression::None,
        }
    }
}

impl From<bool> for Compression {
    fn from(value: bool) -> Self {
        match value {
            true => Compression::Zstd,
            false => Compression::None,
        }
    }
}

/// Read blocks from a flat file reader.
///
/// This function processes flat files that are already loaded into memory, supporting both
/// compressed (Zstd) and uncompressed data. If the data is successfully decoded, it returns a
/// vector of `Block` structs representing the blocks contained within the file. The number of
/// blocks returned depends on the file's content and format, which may include one or more blocks.
///
/// # Arguments
///
/// * `reader`: A readable source of the file contents, implementing the [`Read`] trait.
/// * `compression`: The compression type applied to the flat file's data, if any. Accepts [`Compression::Zstd`]
///   for Zstd-compressed data, or [`Compression::None`] for uncompressed data.
pub fn read_blocks_from_reader<R: Read>(
    reader: R,
    compression: Compression,
) -> Result<Vec<Block>, DecoderError> {
    const CONTENT_TYPE: &str = "ETH";

    let mut file_contents: Box<dyn Read> = match compression {
        Compression::Zstd => Box::new(Cursor::new(zstd::decode_all(reader)?)),
        Compression::None => Box::new(reader),
    };

    let dbin_file = DbinFile::try_from_read(&mut file_contents)?;
    if dbin_file.content_type() != CONTENT_TYPE {
        return Err(DecoderError::ContentTypeInvalid(
            dbin_file.content_type().to_string(),
        ));
    }

    dbin_file
        .into_iter()
        .map(|message| {
            let block = decode_block_from_bytes(&message)?;
            if !block_is_verified(&block) {
                Err(DecoderError::VerificationFailed {
                    block_number: block.number,
                })
            } else {
                Ok(block)
            }
        })
        .collect()
}

fn block_is_verified(block: &Block) -> bool {
    if block.number != 0 {
        if !block.receipt_root_is_verified() {
            error!(
                "Receipt root verification failed for block {}",
                block.number
            );
            return false;
        }

        if !block.transaction_root_is_verified() {
            error!(
                "Transaction root verification failed for block {}",
                block.number
            );
            return false;
        }
    }
    true
}

/// Reader enum to handle different types of readers
///
/// - [`Reader::Buf`]: A [`BufReader`] that reads from a byte slice
/// - [`Reader::StdIn`]: A reader that reads from standard input, with or without compression
#[derive(Debug)]
pub enum Reader {
    /// A [`BufReader`] that reads from a byte slice
    Buf(BufReader<Cursor<Vec<u8>>>),
    /// A reader that reads from standard input, with or without compression
    StdIn(Compression),
}

impl Reader {
    pub(crate) fn into_reader(self) -> Result<Box<dyn Read>, DecoderError> {
        match self {
            Reader::StdIn(compression) => match compression {
                Compression::Zstd => Ok(Box::new(zstd::stream::Decoder::new(std::io::stdin())?)),
                Compression::None => Ok(Box::new(BufReader::with_capacity(
                    // Set buffer size to 128 MB (64 * 2 MB) for reading large data efficiently.
                    // `(64 * 2) << 20` converts 128 MB to bytes (128 * 1,048,576 = 134,217,728 bytes).
                    (64 * 2) << 20,
                    std::io::stdin().lock(),
                ))),
            },
            Reader::Buf(reader) => Ok(Box::new(reader)),
        }
    }
}

impl TryFrom<Reader> for Box<dyn Read> {
    type Error = DecoderError;

    fn try_from(reader: Reader) -> Result<Self, Self::Error> {
        reader.into_reader()
    }
}

/// Set the end block for the range of blocks to read, decode, and verify.
///
/// Enum to handle the end block of the stream.
/// It can be the merge block, i.e. the last pre-merge block, or a specific block number.
pub enum EndBlock {
    /// The last pre-merge block.
    MergeBlock,
    /// A specific block number.
    Block(u64),
}

impl EndBlock {
    fn block_number(&self) -> u64 {
        const LAST_PREMERGE_BLOCK: u64 = 15537393;
        match self {
            EndBlock::MergeBlock => LAST_PREMERGE_BLOCK,
            EndBlock::Block(block_number) => *block_number,
        }
    }
}

impl From<Option<u64>> for EndBlock {
    fn from(value: Option<u64>) -> Self {
        value.map_or(EndBlock::MergeBlock, EndBlock::Block)
    }
}

/// Get an iterator of decoded, verified blocks from a reader.
///
/// Skips invalid blocks and returns an iterator of verified blocks.
///
/// # Arguments
///
/// * `reader`: A [`Reader`] enum that specifies the source of the block data. The reader can be a
///   [`BufReader`] or a `StdIn` reader with or without compression.
/// * `end_block`: Specifies the block number at which to stop streaming. By default, this is set to
///   block 15537393, the last block before the Ethereum merge.
pub fn stream_blocks(
    reader: Reader,
    end_block: EndBlock,
) -> Result<impl Iterator<Item = Block>, DecoderError> {
    let mut current_block_number = 0;

    let mut reader = reader.into_reader()?;
    let end_block = end_block.block_number();

    let mut blocks = Vec::new();

    loop {
        match read_block_from_reader(&mut reader) {
            Ok(message) => {
                match decode_block_from_bytes(&message) {
                    Ok(block) => {
                        current_block_number = block.number;

                        if block_is_verified(&block) {
                            blocks.push(block);
                        } else {
                            info!("Block verification failed, skipping block {}", block.number);
                        }
                    }
                    Err(e) => return Err(e),
                };
            }
            Err(DecoderError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                if current_block_number < end_block {
                    info!("Reached end of file, waiting for more blocks");
                    continue;
                }
                break;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(blocks.into_iter())
}

/// Decodes a block from a byte slice.
#[allow(deprecated)]
fn decode_block_from_bytes(bytes: &[u8]) -> Result<Block, DecoderError> {
    let block_stream = BstreamBlock::decode(bytes)?;
    let block = Block::decode(block_stream.payload_buffer.as_slice())?;
    Ok(block)
}

/// Converts a Parquet file containing block header data (from nozzle) into [`Vec<BlockHeader>`]
/// structs.
///
/// This function reads the given Parquet file, extracts the necessary fields from each row, and
/// constructs a [`BlockHeader`] for each block found in the file. The resulting [`BlockHeader`] structs
/// are returned as a `Vec<BlockHeader>`. This is useful for transforming raw block data from Parquet
/// format into the format expected by the FirehoseProtos system.
pub fn parquet_to_headers(file: File) -> Result<Vec<BlockHeader>, parquet::errors::ParquetError> {
    let reader = SerializedFileReader::new(file)?;

    let iter = reader.get_row_iter(None)?;

    let mut bheaders: Vec<BlockHeader> = Vec::new();
    for row_result in iter {
        let row = row_result.unwrap();

        let bheader = BlockHeader {
            number: row.get_ulong(0).unwrap(),
            parent_hash: row.get_bytes(3)?.data().to_vec(),
            uncle_hash: row.get_bytes(4)?.data().to_vec(),
            coinbase: row.get_bytes(5)?.data().to_vec(),
            state_root: row.get_bytes(6)?.data().to_vec(),
            transactions_root: row.get_bytes(7)?.data().to_vec(),
            receipt_root: row.get_bytes(8)?.data().to_vec(),
            logs_bloom: row.get_bytes(9)?.data().to_vec(),
            difficulty: Some(BigInt {
                bytes: row.get_bytes(10)?.data().to_vec(),
            }),
            // total_difficulty is not present in parquet headers
            total_difficulty: Some(BigInt { bytes: vec![] }),
            gas_limit: row.get_ulong(11).unwrap(),
            gas_used: row.get_ulong(12).unwrap(),
            timestamp: row
                .get_timestamp_micros(1)
                .map(|timestamp_micros| Timestamp {
                    seconds: timestamp_micros / 1_000_000,
                    nanos: (timestamp_micros % 1_000_000) as i32 * 1000, // Convert microseconds to nanoseconds
                })
                .ok(),
            extra_data: row.get_bytes(13)?.data().to_vec(),
            mix_hash: row.get_bytes(15)?.data().to_vec(),
            nonce: row.get_ulong(16).unwrap(),
            hash: row.get_bytes(2)?.data().to_vec(),
            base_fee_per_gas: Some(BigInt {
                bytes: row.get_bytes(16)?.data().to_vec(),
            }),
            // withdrawals_root not present in parquet headers
            withdrawals_root: vec![],
            // tx_dependency is not present in parquet files
            tx_dependency: Some(Uint64NestedArray { val: Vec::new() }),
            blob_gas_used: None,
            excess_blob_gas: None,
            // TODO: does the RPC endpoints provide this data?
            parent_beacon_root: vec![],
        };

        bheaders.push(bheader);
    }
    Ok(bheaders)
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    #[test]
    fn test_read_parquet() {
        let file = File::open("tests/000000000.parquet").unwrap();
        let _ = parquet_to_headers(file);
    }
}
