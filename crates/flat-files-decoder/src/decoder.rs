use std::{
    io::{BufReader, Cursor, Read},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use alloy_primitives::B256;
use firehose_protos::{
    bstream,
    ethereum_v2::{self, Block, BlockHeader},
};
use prost::Message;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{error, info, trace};

use crate::{
    dbin::{DbinFile, DbinHeader},
    error::DecoderError,
};

#[derive(Clone, Copy, Debug, Default)]
pub enum Compression {
    Zstd,
    #[default]
    None,
}

impl From<&str> for Compression {
    fn from(value: &str) -> Self {
        match value {
            "true" | "1" => Compression::Zstd,
            "false" | "0" => Compression::None,
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
/// * `compression`: A boolean indicating whether the input buffer should be decompressed.
pub fn decode_reader<R: Read>(
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
        return Err(DecoderError::DbinContentTypeInvalid(
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

/// A struct to hold the block hash, block number, and total difficulty of a block.
#[derive(Serialize, Deserialize)]
pub struct HeaderRecordWithNumber {
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
                .ok_or(Self::Error::TotalDifficultyInvalid)?
                .bytes
                .clone(),
            block_number: block.number,
        };
        Ok(header_record_with_number)
    }
}

/// Reader enum to handle different types of readers
/// It can be a BufReader or a StdIn reader with or without compression
/// The BufReader is used when the data is already loaded into memory,
/// assuming that the data is not compressed.
#[derive(Debug)]
pub enum Reader {
    Buf(BufReader<Cursor<Vec<u8>>>),
    StdIn(Compression),
}

impl Reader {
    fn into_reader(self) -> Result<Box<dyn Read>, DecoderError> {
        use Reader::*;

        let reader = match self {
            StdIn(compression) => match compression {
                Compression::Zstd => {
                    let reader = zstd::stream::Decoder::new(std::io::stdin())?;
                    Box::new(reader) as Box<dyn Read>
                }
                Compression::None => {
                    let reader = BufReader::with_capacity((64 * 2) << 20, std::io::stdin().lock());
                    Box::new(reader) as Box<dyn Read>
                }
            },
            Buf(reader) => Box::new(reader) as Box<dyn Read>,
        };

        Ok(reader)
    }
}

impl TryFrom<Reader> for Box<dyn Read> {
    type Error = DecoderError;

    fn try_from(reader: Reader) -> Result<Self, Self::Error> {
        reader.into_reader()
    }
}

/// Enum to handle the end block of the stream
/// It can be the merge block or a specific block number
pub enum EndBlock {
    MergeBlock,
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
        match value {
            Some(block_number) => EndBlock::Block(block_number),
            None => EndBlock::MergeBlock,
        }
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
pub async fn stream_blocks(
    reader: Reader,
    end_block: EndBlock,
) -> Result<impl futures::Stream<Item = Block>, DecoderError> {
    let (block_stream_tx, block_stream_rx) = tokio::sync::mpsc::channel::<Block>(8192);
    let (bytes_tx, bytes_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(8192);

    let current_block_number = Arc::new(AtomicU64::new(0));
    let current_block_number_clone = Arc::clone(&current_block_number);

    // THIS ALL NEEDS FIXING
    let end_block = end_block.block_number();

    tokio::spawn(decode_blocks_stream(
        bytes_rx,
        block_stream_tx,
        current_block_number_clone,
    ));

    let mut reader = reader.into_reader()?;

    loop {
        let current_block_number = current_block_number.load(Ordering::SeqCst);

        match DbinHeader::read_message_from_stream(&mut reader) {
            Ok(message) => {
                if let Err(e) = bytes_tx.send(message).await {
                    error!("Error sending message to stream: {e}");
                    break;
                }
            }
            Err(e) => {
                if let DecoderError::Io(ref e) = e {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        if current_block_number < end_block {
                            info!("Reached end of file, waiting for more blocks");
                            continue;
                        } else {
                            info!("All blocks have been read");
                            break;
                        }
                    }
                }

                error!("Error reading dbin file: {}", e);
                break;
            }
        }
    }

    drop(bytes_tx);

    Ok(ReceiverStream::new(block_stream_rx))
}

async fn decode_blocks_stream(
    mut rx: mpsc::Receiver<Vec<u8>>,
    stream_tx: mpsc::Sender<Block>,
    current_block_number: Arc<AtomicU64>,
) {
    while let Some(message) = rx.recv().await {
        trace!("Received message");
        let block = match decode_block_from_bytes(&message) {
            Ok(block) => block,
            Err(e) => {
                error!("Error decoding block: {e}");
                break;
            }
        };

        current_block_number.store(block.number, Ordering::SeqCst);

        let receipts_check_process = spawn_check(&block, |b| {
            if b.receipt_root_is_verified() {
                Ok(())
            } else {
                Err(DecoderError::ReceiptRootInvalid)
            }
        });

        let transactions_check_process = spawn_check(&block, |b| {
            if b.transaction_root_is_verified() {
                Ok(())
            } else {
                Err(DecoderError::TransactionRootInvalid)
            }
        });

        let joint_return = tokio::try_join![receipts_check_process, transactions_check_process];

        if let Err(e) = joint_return {
            error!("{e}");
            break;
        }

        if let Err(e) = stream_tx.send(block).await {
            error!("Error sending block to stream: {e}");
            break;
        }
    }
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
