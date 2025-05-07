// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::io::{BufReader, Cursor, Read};

use firehose_protos::{BstreamBlock, EthBlock as Block, SolBlock};
use prost::Message;
use tracing::{error, info};

use crate::{dbin::read_block_from_reader, error::DecoderError, DbinFile};

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

#[derive(Clone)]
enum AnyBlock {
    Eth(Block),
    Sol(SolBlock),
}

// So far we have parsed .dbin files containing Blocks
// from these chains, but others may be added in the
// future. The content type in the dbin header may also
// vary depending on the version of the file.
#[derive(Clone)]
enum ContentType {
    Eth,
    Sol,
}

impl TryFrom<&str> for ContentType {
    type Error = DecoderError;

    // These are the content types we have so far encountered, but there
    // are others which may be added in the future.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ETH" => Ok(ContentType::Eth),
            "type.googleapis.com/sf.solana.type.v1.Block" => Ok(ContentType::Sol),
            _ => Err(DecoderError::ContentTypeInvalid(value.to_string())),
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
) -> Result<Vec<AnyBlock>, DecoderError> {
    let mut file_contents: Box<dyn Read> = match compression {
        Compression::Zstd => Box::new(Cursor::new(zstd::decode_all(reader)?)),
        Compression::None => Box::new(reader),
    };

    let dbin_file = DbinFile::try_from_read(&mut file_contents)?;
    let content_type: ContentType = dbin_file.content_type().try_into()?;

    dbin_file
        .into_iter()
        .map(|message| {
            let block = decode_block_from_bytes(&message, content_type.clone())?;
            let (verified, number) = block_is_verified(&block);
            if !verified {
                Err(DecoderError::VerificationFailed {
                    block_number: number,
                })
            } else {
                Ok(block)
            }
        })
        .collect()
}

fn block_is_verified(block: &AnyBlock) -> (bool, u64) {
    match block {
        AnyBlock::Eth(eth_block) => {
            let block_number = eth_block.number;
            if block_number != 0 {
                if !eth_block.receipt_root_is_verified() {
                    error!(
                        "Receipt root verification failed for block {}",
                        block_number
                    );
                    return (false, block_number);
                }

                if !eth_block.transaction_root_is_verified() {
                    error!(
                        "Transaction root verification failed for block {}",
                        block_number
                    );
                    return (false, block_number);
                }
            }
            (true, block_number)
        }
        // Logic is not yet implemented for verifying Solana Blocks
        AnyBlock::Sol(sol_block) => {
            let block_number = sol_block.slot;
            (true, block_number)
        }
    }
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

fn decode_block_from_bytes(
    bytes: &[u8],
    content_type: ContentType,
) -> Result<AnyBlock, DecoderError> {
    let block_stream = BstreamBlock::decode(bytes)?;
    let block_stream_payload = block_stream
        .payload
        .map(|p| p.value)
        .unwrap_or(block_stream.payload_buffer);

    match content_type {
        ContentType::Eth => {
            let block = Block::decode(block_stream_payload.as_slice())?;
            Ok(AnyBlock::Eth(block))
        }
        ContentType::Sol => {
            let block = SolBlock::decode(block_stream_payload.as_slice())?;
            Ok(AnyBlock::Sol(block))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};

    use tracing_subscriber::field::debug;

    use super::*;

    #[test]
    fn test_read_eth_blocks_from_reader() {
        let file = File::open("tests/0000000000.dbin").unwrap();
        let mut reader = BufReader::new(file);

        let block = read_blocks_from_reader(&mut reader, false.into()).unwrap();
    }

    #[test]
    fn test_read_sol_blocks_from_reader() {
        let file = File::open("tests/0325942300.dbin.zst").unwrap();
        let mut reader = BufReader::new(file);

        let block = read_blocks_from_reader(&mut reader, true.into()).unwrap();
    }
}
