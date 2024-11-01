use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Cursor, Read, Write},
    path::PathBuf,
};

use alloy_primitives::B256;
use clap::Parser;
use firehose_protos::{
    bstream,
    ethereum_v2::{self, Block, BlockHeader},
};
use prost::Message;
use serde::{Deserialize, Serialize};
use tokio::join;
use tracing::{error, info, trace};
use zstd::stream::decode_all;

use crate::{
    cli::{Cli, Commands},
    dbin::DbinFile,
    decompression::Decompression,
    error::DecoderError,
};

pub async fn run() -> Result<(), DecoderError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Stream {
            decompression,
            end_block,
        } => match decompression {
            Decompression::Zstd => {
                let reader = zstd::stream::Decoder::new(io::stdin())?;
                let writer = BufWriter::new(io::stdout().lock());
                stream_blocks(reader, writer, end_block).await
            }
            Decompression::None => {
                let reader = BufReader::with_capacity((64 * 2) << 20, io::stdin().lock());
                let writer = BufWriter::new(io::stdout().lock());
                stream_blocks(reader, writer, end_block).await
            }
        },
        Commands::Decode {
            input,
            headers_dir,
            output,
            decompression,
        } => {
            let blocks = decode_flat_files(
                input,
                output.as_deref(),
                headers_dir.as_deref(),
                decompression,
            )?;

            info!("Total blocks: {}", blocks.len());

            Ok(())
        }
    }
}

/// Decodes and optionally verifies block flat files from a given directory or single file.
///
/// This function processes input which can be a file or a directory containing multiple `.dbin` files.
/// If `headers_dir` is provided, it verifies the block headers against the files found in this directory.
/// These header files must be in JSON format and named after the block number they represent (e.g., `block-<block number>.json`).
/// it can also handle `zstd` compressed flat files.
///
/// # Arguments
///
/// * `input`: A [`String`] specifying the path to the input directory or file.
/// * `output`: An [`Option<&str>`] specifying the directory where decoded blocks should be written.
///             If `None`, decoded blocks are not written to disk.
/// * `headers_dir`: An [`Option<&str>`] specifying the directory containing header files for verification.
///                  Must be a directory if provided.
/// * `decompress`: A [`Decompression`] enum specifying if it is necessary to decompress from zstd.
pub fn decode_flat_files(
    input: String,
    output: Option<&str>,
    headers_dir: Option<&str>,
    decompress: Decompression,
) -> Result<Vec<Block>, DecoderError> {
    let metadata = fs::metadata(&input)?;

    if let Some(output) = output {
        fs::create_dir_all(output)?;
    }

    if metadata.is_dir() {
        decode_flat_files_dir(&input, output, headers_dir, decompress)
    } else if metadata.is_file() {
        handle_file(&PathBuf::from(input), output, headers_dir, decompress)
    } else {
        Err(DecoderError::InvalidInput)
    }
}

fn decode_flat_files_dir(
    input: &str,
    output: Option<&str>,
    headers_dir: Option<&str>,
    decompress: Decompression,
) -> Result<Vec<Block>, DecoderError> {
    info!("Processing directory: {}", input);
    let paths = fs::read_dir(input)?;

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
        match handle_file(&path.path(), output, headers_dir, decompress) {
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

/// Decodes and optionally verifies block flat files from a single file.
///
/// This function decodes flat files and, if an `output` directory is provided, writes the decoded blocks to this directory.
/// If no `output` is specified, the decoded blocks are not written to disk. The function can also verify block headers
/// against header files found in an optional `headers_dir`. These header files must be in JSON format and named after
/// the block number they represent (e.g., `block-<block number>.json`). Additionally, the function supports handling `zstd` compressed
/// flat files if decompression is required.
///
/// # Arguments
///
/// * `input`: A [`String`] specifying the path to the file.
/// * `output`: An [`Option<&str>`] specifying the directory where decoded blocks should be written.
///             If `None`, decoded blocks are not written to disk.
/// * `headers_dir`: An [`Option<&str>`] specifying the directory containing header files for verification.
///                  Must be a directory if provided.
/// * `decompress`: A [`Decompression`] enum indicating whether decompression from `zstd` format is necessary.
///
pub fn handle_file(
    path: &PathBuf,
    output: Option<&str>,
    headers_dir: Option<&str>,
    decompress: Decompression,
) -> Result<Vec<Block>, DecoderError> {
    let input_file = BufReader::new(File::open(path)?);

    // Check if decompression is required and read the file accordingly.
    let mut file_contents: Box<dyn Read> = match decompress {
        Decompression::Zstd => {
            let decompressed_data = decode_all(input_file)?;
            Box::new(Cursor::new(decompressed_data))
        }
        Decompression::None => Box::new(input_file),
    };

    let dbin_file = DbinFile::try_from_read(&mut file_contents)?;
    if dbin_file.header.content_type != "ETH" {
        return Err(DecoderError::InvalidContentType(
            dbin_file.header.content_type,
        ));
    }

    let mut blocks: Vec<Block> = vec![];

    for message in dbin_file.messages {
        blocks.push(handle_block(&message, output, headers_dir)?);
    }

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
pub fn handle_buf(buf: &[u8], decompress: Decompression) -> Result<Vec<Block>, DecoderError> {
    let buf = match decompress {
        Decompression::Zstd => zstd::decode_all(buf)?,
        Decompression::None => buf.to_vec(),
    };

    let dbin_file = DbinFile::try_from_read(&mut Cursor::new(buf))?;

    let mut blocks: Vec<Block> = vec![];

    for message in dbin_file.messages {
        blocks.push(handle_block(&message, None, None)?);
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

fn handle_block(
    message: &Vec<u8>,
    output: Option<&str>,
    headers_dir: Option<&str>,
) -> Result<Block, DecoderError> {
    let block = decode_block_from_bytes(message)?;

    if let Some(headers_dir) = headers_dir {
        let header_file_path = format!("{}/{}.json", headers_dir, block.number);
        let header_file = File::open(header_file_path)?;
        let header_roots: BlockHeaderRoots = serde_json::from_reader(header_file)?;

        if !header_roots.block_header_matches(&block) {
            return Err(DecoderError::MatchRootsFailed {
                block_number: block.number,
            });
        }
    }

    if block.number != 0 {
        if !block.receipt_root_is_verified() {
            return Err(DecoderError::ReceiptRoot);
        }

        if !block.transaction_root_is_verified() {
            return Err(DecoderError::TransactionRoot);
        }
    }

    if let Some(output) = output {
        let file_name = format!("{}/block-{}.json", output, block.number);
        let mut out_file = File::create(file_name)?;

        let block_json = serde_json::to_string(&block)?;

        out_file.write_all(block_json.as_bytes())?;
    }

    Ok(block)
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

fn decode_block_from_bytes(bytes: &Vec<u8>) -> Result<Block, DecoderError> {
    let block_stream = bstream::v1::Block::decode(bytes.as_slice())?;
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

#[cfg(test)]
mod tests {
    use firehose_protos::bstream::v1::Block as BstreamBlock;
    use std::io::{BufReader, BufWriter};

    use super::*;

    const TEST_ASSET_PATH: &str = "../../test-assets";

    #[test]
    fn test_handle_file() {
        let path = PathBuf::from(format!("{TEST_ASSET_PATH}/example0017686312.dbin"));

        let result = handle_file(&path, None, None, Decompression::None);

        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_file_zstd() {
        let path = PathBuf::from(format!("{TEST_ASSET_PATH}/0000000000.dbin.zst"));

        let result = handle_file(&path, None, None, Decompression::Zstd);

        assert!(result.is_ok());
        let blocks: Vec<Block> = result.unwrap();
        assert_eq!(blocks[0].number, 0);
    }

    #[test]
    fn test_check_valid_root_fail() {
        let path = PathBuf::from(format!("{TEST_ASSET_PATH}/example0017686312.dbin"));
        let mut file = BufReader::new(File::open(path).expect("Failed to open file"));
        let dbin_file: DbinFile =
            DbinFile::try_from_read(&mut file).expect("Failed to parse dbin file");

        let message = dbin_file.messages[0].clone();

        let block_stream = BstreamBlock::decode(message.as_slice()).unwrap();
        let mut block = Block::decode(block_stream.payload_buffer.as_slice()).unwrap();

        assert!(block.receipt_root_is_verified());

        // Remove an item from the block to make the receipt root invalid
        block.transaction_traces.pop();

        assert!(!block.receipt_root_is_verified());
    }

    #[test]
    fn test_block_stream() {
        let mut buffer = Vec::new();
        let cursor: Cursor<&mut Vec<u8>> = Cursor::new(&mut buffer);
        let inputs = vec![
            format!("{TEST_ASSET_PATH}/example-create-17686085.dbin"),
            format!("{TEST_ASSET_PATH}/example0017686312.dbin"),
        ];
        {
            let mut writer = BufWriter::new(cursor);
            for i in inputs {
                let mut input = File::open(i).expect("couldn't read input file");

                std::io::copy(&mut input, &mut writer).expect("couldn't copy");
                writer.flush().expect("failed to flush output");
            }
        }
        let mut cursor = Cursor::new(&buffer);
        cursor.set_position(0);

        let reader = BufReader::new(cursor);
        let mut in_buffer = Vec::new();
        let writer = BufWriter::new(Cursor::new(&mut in_buffer));

        matches!(
            tokio_test::block_on(stream_blocks(reader, writer, None)),
            Ok(())
        );
    }

    #[test]
    fn test_handle_buff() {
        let path = PathBuf::from(format!("{TEST_ASSET_PATH}/example0017686312.dbin"));
        let file = BufReader::new(File::open(path).expect("Failed to open file"));
        let mut reader = BufReader::new(file);

        let mut buffer = Vec::new();

        reader
            .read_to_end(&mut buffer)
            .expect("Failed to read file");

        let result = handle_buf(&buffer, Decompression::None);
        if let Err(e) = result {
            panic!("handle_buf failed: {}", e);
        }
        assert!(result.is_ok(), "handle_buf should complete successfully");
    }

    #[test]
    fn test_handle_buff_decompress() {
        let path = PathBuf::from(format!("{TEST_ASSET_PATH}/0000000000.dbin.zst"));
        let file = BufReader::new(File::open(path).expect("Failed to open file"));
        let mut reader = BufReader::new(file);

        let mut buffer = Vec::new();

        reader
            .read_to_end(&mut buffer)
            .expect("Failed to read file");

        let result = handle_buf(&buffer, Decompression::Zstd);
        assert!(
            result.is_ok(),
            "handle_buf should complete successfully with decompression"
        );
    }
}
