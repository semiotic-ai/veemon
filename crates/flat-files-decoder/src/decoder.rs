use std::{
    fs::{File, ReadDir},
    io::{BufReader, Cursor, Read, Write},
    path::PathBuf,
};

use alloy_primitives::B256;
use firehose_protos::{
    bstream,
    ethereum_v2::{self, Block, BlockHeader},
};
use prost::Message;
use serde::{Deserialize, Serialize};
use tokio::join;
use tracing::{error, info, trace};

use crate::{compression::Compression, dbin::DbinFile, error::DecoderError};

pub fn read_flat_files(
    paths: ReadDir,
    compression: Compression,
) -> Result<Vec<Block>, DecoderError> {
    let mut blocks: Vec<Block> = vec![];
    for path in paths {
        let path = path?;
        match path.path().extension() {
            Some(ext) => {
                if ext != "dbin" {
                    continue;
                }
            }
            None => continue,
        };

        trace!("Processing file: {}", path.path().display());
        match read_flat_file(&path.path(), compression) {
            Ok(file_blocks) => {
                blocks.extend(file_blocks);
            }
            Err(err) => {
                error!("Failed to process file: {}", err);
            }
        }
    }

    Ok(blocks)
}

/// Decodes and verifies block flat files from a single file.
///
/// This function decodes and verifies blocks contained within flat files.
/// Additionally, the function supports handling `zstd` compressed flat files if decompression is required.
///
/// # Arguments
///
/// * `input`: A [`String`] specifying the path to the file.
/// * `decompress`: A [`Decompression`] enum indicating whether decompression from `zstd` format is necessary.
///
pub fn read_flat_file(
    path: &PathBuf,
    compression: Compression,
) -> Result<Vec<Block>, DecoderError> {
    let input_file = BufReader::new(File::open(path)?);

    let blocks = handle_buffer(input_file, compression)?;

    Ok(blocks)
}

/// Decodes a flat file from a buffer containing its contents and optionally decompresses it.
///
/// Decodes flat files that are already loaded into memory, without direct file system access.
/// It can handle both compressed (if `zstd` decompression is specified) and uncompressed data. Upon successful
/// decoding, it returns a vector of all the blocks contained within the flat file. The actual number of blocks
/// returned depends on the format and content of the flat file—ranging from a single block to multiple blocks.
///
/// # Arguments
///
/// * `buf`: A byte slice referencing the in-memory content of the flat file to be decoded.
/// * `decompress`: A boolean indicating whether the input buffer should be decompressed.
///
pub fn handle_buffer<R: Read>(
    reader: R,
    compression: Compression,
) -> Result<Vec<Block>, DecoderError> {
    const CONTENT_TYPE: &str = "ETH";

    let mut file_contents: Box<dyn Read> = match compression {
        Compression::Zstd => {
            let decompressed_data = zstd::decode_all(reader)?;
            Box::new(Cursor::new(decompressed_data))
        }
        Compression::None => Box::new(reader),
    };

    let dbin_file = DbinFile::try_from_read(&mut file_contents)?;
    if dbin_file.header.content_type != CONTENT_TYPE {
        return Err(DecoderError::InvalidContentType(
            dbin_file.header.content_type,
        ));
    }

    let mut blocks: Vec<Block> = vec![];

    for message in dbin_file.messages {
        let block = decode_block_from_bytes(&message)?;

        if !block_is_verified(&block) {
            return Err(DecoderError::VerificationFailed {
                block_number: block.number,
            });
        }

        blocks.push(block);
    }

    Ok(blocks)
}

/// A struct to hold the receipt and transactions root for a `Block`.
/// This struct is used to compare the receipt and transactions roots of a block
/// with the receipt and transactions roots of another block.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeaderRoots {
    receipt_root: B256,
    transactions_root: B256,
}

impl TryFrom<&Block> for BlockHeaderRoots {
    type Error = DecoderError;

    fn try_from(block: &Block) -> Result<Self, Self::Error> {
        block.header()?.try_into()
    }
}

impl TryFrom<&BlockHeader> for BlockHeaderRoots {
    type Error = DecoderError;

    fn try_from(header: &BlockHeader) -> Result<Self, Self::Error> {
        let receipt_root: [u8; 32] = header.receipt_root.as_slice().try_into()?;
        let transactions_root: [u8; 32] = header.transactions_root.as_slice().try_into()?;

        Ok(Self {
            receipt_root: receipt_root.into(),
            transactions_root: transactions_root.into(),
        })
    }
}

impl BlockHeaderRoots {
    /// Checks if the receipt and transactions roots of a block header match the receipt and transactions roots of another block.
    pub fn block_header_matches(&self, block: &Block) -> bool {
        let block_header_roots = match block.try_into() {
            Ok(block_header_roots) => block_header_roots,
            Err(e) => {
                error!("Failed to convert block to header roots: {e}");
                return false;
            }
        };

        self.block_header_roots_match(&block_header_roots)
    }

    fn block_header_roots_match(&self, block_header_roots: &BlockHeaderRoots) -> bool {
        self == block_header_roots
    }
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

#[derive(Serialize, Deserialize)]
struct HeaderRecordWithNumber {
    block_hash: Vec<u8>,
    block_number: u64,
    total_difficulty: Vec<u8>,
}

impl TryFrom<&Block> for HeaderRecordWithNumber {
    type Error = DecoderError;

    fn try_from(block: &Block) -> Result<Self, Self::Error> {
        let block_header = block.header()?;

        let header_record_with_number = HeaderRecordWithNumber {
            block_hash: block.hash.clone(),
            total_difficulty: block_header
                .total_difficulty
                .as_ref()
                .ok_or(Self::Error::InvalidTotalDifficulty)?
                .bytes
                .clone(),
            block_number: block.number,
        };
        Ok(header_record_with_number)
    }
}

/// Decode blocks from a reader and writes them, serialized, to a writer
///
/// data can be piped into this function from stdin via `cargo run stream < ./example0017686312.dbin`.
/// It also has a check for end_block. By default, it stops the stream reading when MERGE_BLOCK
/// is reached.
///
/// # Arguments
///
/// * `end_block`: For blocks after the merge, Ethereum sync committee should be used. This is why the default block
///   for this param is the LAST_PREMERGE_BLOCK (block 15537393)
/// * `reader`: where bytes are read from
/// * `writer`: where bytes written to
pub async fn stream_blocks<R: Read, W: Write>(
    mut reader: R,
    mut writer: W,
    end_block: Option<usize>,
) -> Result<(), DecoderError> {
    const LAST_PREMERGE_BLOCK: usize = 15537393;

    let end_block = match end_block {
        Some(end_block) => end_block,
        None => LAST_PREMERGE_BLOCK,
    };

    let mut block_number = 0;

    loop {
        match DbinFile::read_message_stream(&mut reader) {
            Ok(message) => {
                let block = decode_block_from_bytes(&message)?;

                block_number = block.number as usize;

                let receipts_check_process =
                    spawn_check(&block, |b| match b.receipt_root_is_verified() {
                        true => Ok(()),
                        false => Err(DecoderError::ReceiptRoot),
                    });

                let transactions_check_process =
                    spawn_check(&block, |b| match b.transaction_root_is_verified() {
                        true => Ok(()),
                        false => Err(DecoderError::TransactionRoot),
                    });

                let joint_return = join![receipts_check_process, transactions_check_process];
                joint_return.0?;
                joint_return.1?;

                let header_record_with_number = HeaderRecordWithNumber::try_from(&block)?;
                let header_record_bin = bincode::serialize(&header_record_with_number)?;

                let size = header_record_bin.len() as u32;
                writer.write_all(&size.to_be_bytes())?;
                writer.write_all(&header_record_bin)?;
                writer.flush()?;
            }
            Err(e) => {
                if let DecoderError::Io(ref e) = e {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        if block_number < end_block {
                            info!("Reached end of file, waiting for more blocks");
                            // More blocks to read
                            continue;
                        } else {
                            // All blocks have been read
                            break;
                        }
                    }
                }

                error!("Error reading dbin file: {}", e);
                break;
            }
        }
    }
    Ok(())
}

/// Decodes a block from a byte slice.
fn decode_block_from_bytes(bytes: &[u8]) -> Result<Block, DecoderError> {
    let block_stream = bstream::v1::Block::decode(bytes)?;
    let block = ethereum_v2::Block::decode(block_stream.payload_buffer.as_slice())?;
    Ok(block)
}

// Define a generic function to spawn a blocking task for a given check.
fn spawn_check<F>(block: &Block, check: F) -> tokio::task::JoinHandle<()>
where
    F: FnOnce(&Block) -> Result<(), DecoderError> + Send + 'static,
{
    let block_clone = block.clone();
    tokio::task::spawn_blocking(move || match check(&block_clone) {
        Ok(_) => {}
        Err(err) => {
            error!("{}", err);
        }
    })
}
