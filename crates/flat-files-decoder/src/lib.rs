//! # Flat File decoder for Firehose
//! Crate that provides utility functions to read and verify flat files from disk.
//! The verifier currently matches computed receipts & transaction roots against the roots
//! provided in the block header. Optionally, the verifier can also check the block headers
//! against a directory of block headers in json format.

pub mod dbin;
pub mod error;
pub mod headers;
pub mod receipts;
pub mod transactions;

use crate::error::DecodeError;
use crate::headers::check_valid_header;
use crate::transactions::check_transaction_root;
use dbin::DbinFile;
use error::CheckError;
use headers::HeaderRecordWithNumber;
use prost::Message;
use rayon::prelude::*;
use receipts::check_receipt_root;
use sf::ethereum::r#type::v2::Block;
use simple_log::log;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Write};
use std::path::PathBuf;
use tokio::join;

pub mod sf {
    pub mod ethereum {
        pub mod r#type {
            pub mod v2 {
                include!(concat!(env!("OUT_DIR"), "/sf.ethereum.r#type.v2.rs"));
            }
        }
    }
    pub mod bstream {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/sf.bstream.v1.rs"));
        }
    }
}

const MERGE_BLOCK: usize = 15537393;

pub enum DecodeInput {
    Path(String),
    Reader(Box<dyn Read>),
}
/**
* Decode & verify flat files from a directory or a single file.
* Input can be a directory or a file.
* headers_dir is optional but must be a directory if provided.
* If headers_dir is provided, the block headers will be verified against the files in the directory.
* Header files must be named after the block number they represent and be in json format (e.g. 123.json).
* If input is a directory, all files with the extension .dbin will be processed.
* If output is provided, the decoded blocks will be written to the directory.
* If output is not provided, the decoded blocks will not be written to disk.
**/
pub fn decode_flat_files(
    input: String,
    output: Option<&str>,
    headers_dir: Option<&str>,
) -> Result<Vec<Block>, DecodeError> {
    let metadata = fs::metadata(&input).map_err(DecodeError::IoError)?;

    if let Some(output) = output {
        fs::create_dir_all(output).map_err(DecodeError::IoError)?;
    }

    if metadata.is_dir() {
        decode_flat_files_dir(&input, output, headers_dir)
    } else if metadata.is_file() {
        handle_file(&PathBuf::from(input), output, headers_dir)
    } else {
        Err(DecodeError::InvalidInput)
    }
}

fn decode_flat_files_dir(
    input: &str,
    output: Option<&str>,
    headers_dir: Option<&str>,
) -> Result<Vec<Block>, DecodeError> {
    let paths = fs::read_dir(input).map_err(DecodeError::IoError)?;

    let mut blocks: Vec<Block> = vec![];
    for path in paths {
        let path = path.map_err(DecodeError::IoError)?;
        match path.path().extension() {
            Some(ext) => {
                if ext != "dbin" {
                    continue;
                }
            }
            None => continue,
        };

        println!("Processing file: {}", path.path().display());
        match handle_file(&path.path(), output, headers_dir) {
            Ok(file_blocks) => {
                blocks.extend(file_blocks);
            }
            Err(err) => {
                println!("Failed to process file: {}", err);
            }
        }
    }

    Ok(blocks)
}

/**
* Decode & verify a single flat file.
* If output is provided, the decoded blocks will be written to the directory.
* If output is not provided, the decoded blocks will not be written to disk.
* headers_dir is optional but must be a directory if provided.
* If headers_dir is provided, the block headers will be verified against the files in the directory.
* Header files must be named after the block number they represent and be in json format. (e.g. 123.json)
**/
pub fn handle_file(
    path: &PathBuf,
    output: Option<&str>,
    headers_dir: Option<&str>,
) -> Result<Vec<Block>, DecodeError> {
    let mut input_file = BufReader::new(File::open(path).map_err(DecodeError::IoError)?);
    let dbin_file = DbinFile::try_from_read(&mut input_file)?;
    if dbin_file.content_type != "ETH" {
        return Err(DecodeError::InvalidContentType(dbin_file.content_type));
    }

    let mut blocks: Vec<Block> = vec![];

    for message in dbin_file.messages {
        blocks.push(handle_block(&message, output, headers_dir)?);
    }

    Ok(blocks)
}

/**
* Decode & verify a single flat file from a buffer with its contents.
* This is useful for decoding a file that is already in memory.
* Returns a vector of all the blocks in the flat file
* (it can be a single block or 100 blocks depending on format).
**/
pub fn handle_buf(buf: &[u8]) -> Result<Vec<Block>, DecodeError> {
    let dbin_file = DbinFile::try_from_read(&mut Cursor::new(buf))?;

    let mut blocks: Vec<Block> = vec![];

    for message in dbin_file.messages {
        blocks.push(handle_block(&message, None, None)?);
    }
    Ok(blocks)
}

fn handle_block(
    message: &Vec<u8>,
    output: Option<&str>,
    headers_dir: Option<&str>,
) -> Result<Block, DecodeError> {
    let block = decode_block_from_bytes(message)?;

    if let Some(headers_dir) = headers_dir {
        check_valid_header(&block, headers_dir)?;
    }
    if block.number != 0 {
        check_receipt_root(&block)?;
        check_transaction_root(&block)?;
    }

    if let Some(output) = output {
        let file_name = format!("{}/block-{}.json", output, block.number);
        let mut out_file = File::create(file_name)?;

        let block_json = serde_json::to_string(&block)
            .map_err(|err| DecodeError::ProtobufError(err.to_string()))?;

        out_file
            .write_all(block_json.as_bytes())
            .map_err(DecodeError::IoError)?;
    }

    Ok(block)
}

/// Gets a vector of blocks from a single .dbin file
pub fn extract_blocks<R: Read>(mut reader: R) -> Result<Vec<Block>, DecodeError> {
    log::debug!("Reading messages");
    let dbin_file = DbinFile::try_from_read(&mut reader)?;
    log::debug!("Validating blocks");

    // Parallel processing of block headers
    dbin_file
        .messages
        .par_iter()
        .map(|message| handle_block(message, None, None))
        .collect()
}

/// Decode blocks from a reader and writes them, serialized, to a writer
///
/// data can be piped into this function from stdin via `cargo run stream < ./example0017686312.dbin`.
/// It also has a check for end_block. By default, it stops the stream reading when MERGE_BLOCK
/// is reached.
///
/// # Arguments
///
/// * `end_block`: Header Accumulator solution is expensive. For blocks after the merge,
/// Ethereum consensus should be used  in this scenario. This zis why the default block
/// for this variable is the MERGE_BLOCK (block 15537393)
/// * `reader`: where bytes are read from
/// * `writer`: where bytes written to
pub async fn stream_blocks<R: Read, W: Write>(
    mut reader: R,
    mut writer: W,
    end_block: Option<usize>,
) -> Result<(), DecodeError> {
    let end_block = match end_block {
        Some(end_block) => end_block,
        None => MERGE_BLOCK,
    };
    let mut block_number = 0;
    loop {
        match DbinFile::read_message_stream(&mut reader) {
            Ok(message) => {
                let block = decode_block_from_bytes(&message)?;
                block_number = block.number as usize;

                let receipts_check_process = spawn_check(&block, |b| {
                    check_receipt_root(b).map_err(|e| CheckError::ReceiptError(e))
                });

                let transactions_check_process = spawn_check(&block, |b| {
                    check_transaction_root(b).map_err(|e| CheckError::TransactionError(e))
                });

                let joint_return = join![receipts_check_process, transactions_check_process];
                joint_return.0.map_err(|err| DecodeError::JoinError(err))?;
                joint_return.1.map_err(|err| DecodeError::JoinError(err))?;

                let header_record_with_number = HeaderRecordWithNumber::try_from(block)?;
                let header_record_bin = bincode::serialize(&header_record_with_number)
                    .map_err(|err| DecodeError::ProtobufError(err.to_string()))?;

                let size = header_record_bin.len() as u32;
                writer.write_all(&size.to_be_bytes())?;
                writer.write_all(&header_record_bin)?;
                writer.flush().map_err(DecodeError::IoError)?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                if block_number < end_block {
                    log::info!("Reached end of file, waiting for more blocks");
                    continue; // More blocks to read
                } else {
                    break; // read all the blocks
                }
            }
            Err(e) => {
                log::error!("Error reading DBIN file: {}", e);
                break;
            }
        }
    }
    Ok(())
}

fn decode_block_from_bytes(bytes: &Vec<u8>) -> Result<Block, DecodeError> {
    let block_stream = sf::bstream::v1::Block::decode(bytes.as_slice())
        .map_err(|err| DecodeError::ProtobufError(err.to_string()))?;
    let block = sf::ethereum::r#type::v2::Block::decode(block_stream.payload_buffer.as_slice())
        .map_err(|err| DecodeError::ProtobufError(err.to_string()))?;
    Ok(block)
}

// Define a generic function to spawn a blocking task for a given check.
fn spawn_check<F>(block: &Block, check: F) -> tokio::task::JoinHandle<()>
where
    F: FnOnce(&Block) -> Result<(), CheckError> + Send + 'static,
{
    let block_clone = block.clone();
    tokio::task::spawn_blocking(move || match check(&block_clone) {
        Ok(_) => {}
        Err(err) => {
            log::error!("{}", err);
        }
    })
}

#[cfg(test)]
mod tests {
    use prost::Message;

    use crate::dbin::DbinFile;
    use crate::receipts::check_receipt_root;
    use crate::sf::bstream::v1::Block as BstreamBlock;
    use crate::sf::ethereum::r#type::v2::Block;
    use crate::{handle_file, receipts, stream_blocks};
    use std::fs::File;
    use std::io::{self, Cursor, Write};
    use std::io::{BufReader, BufWriter};
    use std::path::PathBuf;

    #[test]
    fn test_handle_file() {
        let path = PathBuf::from("example0017686312.dbin");

        let result = handle_file(&path, None, None);

        assert!(result.is_ok());
    }

    #[test]
    fn test_check_valid_root_fail() {
        let path = PathBuf::from("example0017686312.dbin");
        let mut file = BufReader::new(File::open(path).expect("Failed to open file"));
        let dbin_file: DbinFile =
            DbinFile::try_from_read(&mut file).expect("Failed to parse dbin file");

        let message = dbin_file.messages[0].clone();

        let block_stream = BstreamBlock::decode(message.as_slice()).unwrap();
        let mut block = Block::decode(block_stream.payload_buffer.as_slice()).unwrap();

        block.balance_changes.pop();

        let result = check_receipt_root(&block);
        matches!(
            result,
            Err(receipts::error::ReceiptError::MismatchedRoot(_, _))
        );
    }

    #[test]
    fn test_block_stream() {
        let mut buffer = Vec::new();
        let cursor: Cursor<&mut Vec<u8>> = Cursor::new(&mut buffer);
        let inputs = vec!["example-create-17686085.dbin", "example0017686312.dbin"];
        {
            let mut writer = BufWriter::new(cursor);
            for i in inputs {
                let mut input = File::open(i).expect("couldn't read input file");

                io::copy(&mut input, &mut writer).expect("couldn't copy");
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
}
