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
use tracing::{error, info};

use crate::{
    dbin::{read_message_from_stream, DbinFile},
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

/// Decodes a flat file from an in-memory reader, optionally decompressing it if specified.
///
/// This function processes flat files that are already loaded into memory, supporting both
/// compressed (Zstd) and uncompressed data. If the data is successfully decoded, it returns a
/// vector of `Block` structs representing the blocks contained within the file. The number of
/// blocks returned depends on the file's content and format, which may include one or more blocks.
///
/// # Arguments
///
/// * `reader`: A readable source of the file contents, implementing the `Read` trait.
/// * `compression`: The compression type applied to the flat file's data, if any. Accepts `Compression::Zstd`
///   for Zstd-compressed data, or `Compression::None` for uncompressed data.
pub fn decode_reader<R: Read>(
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
        return Err(DecoderError::DbinContentTypeInvalid(
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
        match block.try_into() {
            Ok(other) => self == &other,
            Err(e) => {
                error!("Failed to convert block to header roots: {e}");
                false
            }
        }
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
        Ok(HeaderRecordWithNumber {
            block_hash: block.hash.clone(),
            block_number: block.number,
            total_difficulty: block
                .header()?
                .total_difficulty
                .as_ref()
                .ok_or(Self::Error::TotalDifficultyInvalid)?
                .bytes
                .clone(),
        })
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
        match self {
            Reader::StdIn(compression) => match compression {
                Compression::Zstd => Ok(Box::new(zstd::stream::Decoder::new(std::io::stdin())?)),
                Compression::None => Ok(Box::new(BufReader::with_capacity(
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
        value.map_or(EndBlock::MergeBlock, EndBlock::Block)
    }
}

/// Streams decoded blocks from an input reader, stopping when a specified end block is reached.
///
/// This asynchronous function continuously reads block data from the provided reader, decoding
/// each block and streaming it as [`Block`] items. It supports handling incoming data in chunks
/// and sends each decoded block through a stream channel, allowing for efficient, concurrent processing.
///
/// The function will continue reading until it reaches an `UnexpectedEof` error, which may indicate
/// the end of the file, or until it has processed the specified `end_block`. If the end of the file
/// is reached before `end_block`, it will wait for more blocks to be available, allowing for a continuous
/// stream when data is appended.
///
/// # Arguments
///
/// * `reader`: A [`Reader`] enum that specifies the source of the block data. The reader can be a
///   [`BufReader`] or a `StdIn` reader with or without compression.
/// * `end_block`: Specifies the block number at which to stop streaming. By default, this is set to
///   `LAST_PREMERGE_BLOCK` (block 15537393), which marks the last block before the Ethereum merge.
///
/// # Returns
///
/// Returns a stream ([`futures::Stream`]) of [`Block`] items. Each item in the stream represents a decoded
/// block from the input data.
pub async fn stream_blocks(
    reader: Reader,
    end_block: EndBlock,
) -> Result<impl futures::Stream<Item = Block>, DecoderError> {
    let (block_stream_tx, block_stream_rx) = tokio::sync::mpsc::channel::<Block>(8192);
    let (bytes_tx, bytes_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(8192);
    let current_block_number = Arc::new(AtomicU64::new(0));

    tokio::spawn(decode_blocks_stream(
        bytes_rx,
        block_stream_tx,
        Arc::clone(&current_block_number),
    ));

    let mut reader = reader.into_reader()?;
    let end_block = end_block.block_number();

    loop {
        match read_message_from_stream(&mut reader) {
            Ok(message) => bytes_tx.send(message).await?,
            Err(DecoderError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                if current_block_number.load(Ordering::SeqCst) < end_block {
                    info!("Reached end of file, waiting for more blocks");
                    continue;
                }
                break;
            }
            Err(e) => return Err(e),
        }
    }

    drop(bytes_tx);
    Ok(ReceiverStream::new(block_stream_rx))
}

async fn decode_blocks_stream(
    mut rx: mpsc::Receiver<Vec<u8>>,
    stream_tx: mpsc::Sender<Block>,
    current_block_number: Arc<AtomicU64>,
) -> Result<(), DecoderError> {
    while let Some(message) = rx.recv().await {
        match decode_block_from_bytes(&message) {
            Ok(block) => {
                current_block_number.store(block.number, Ordering::SeqCst);

                let block = Arc::new(block);

                if spawn_checks(&block).await.is_ok() {
                    stream_tx
                        .send(Arc::try_unwrap(block).unwrap()) // Safe to unwrap since we have a single reference.
                        .await
                        .unwrap();
                }
            }
            Err(e) => {
                return Err(e);
            }
        };
    }

    Ok(())
}

/// Decodes a block from a byte slice.
fn decode_block_from_bytes(bytes: &[u8]) -> Result<Block, DecoderError> {
    let block_stream = bstream::v1::Block::decode(bytes)?;
    let block = ethereum_v2::Block::decode(block_stream.payload_buffer.as_slice())?;
    Ok(block)
}

async fn spawn_checks(block: &Arc<Block>) -> Result<((), ()), DecoderError> {
    Ok(tokio::try_join!(
        verify_async(
            block.clone(),
            |b| b.receipt_root_is_verified(),
            DecoderError::ReceiptRootInvalid
        ),
        verify_async(
            block.clone(),
            |b| b.transaction_root_is_verified(),
            DecoderError::TransactionRootInvalid
        ),
    )?)
}

async fn verify_async<F>(
    block: Arc<Block>,
    check: F,
    error: DecoderError,
) -> Result<(), DecoderError>
where
    F: FnOnce(&Block) -> bool + Send + 'static,
{
    tokio::task::spawn_blocking(move || if check(&block) { Ok(()) } else { Err(error) })
        .await
        .unwrap_or_else(|e| Err(e.into()))
}
