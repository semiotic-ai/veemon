mod dbin;
pub mod error;
mod protos;
mod receipts;
mod transactions;

use crate::error::DecodeError;
use crate::transactions::check_transaction_root;
use dbin::DbinFile;
use protobuf::Message;
use protos::block::Block;
use receipts::check_receipt_root;
use std::fs;
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::PathBuf;

pub fn decode_flat_files(input: &str, output: Option<&str>) -> Result<Vec<Block>, DecodeError> {
    let metadata = fs::metadata(input).map_err(DecodeError::IoError)?;

    if let Some(output) = output {
        fs::create_dir_all(output).map_err(DecodeError::IoError)?;
    }

    if metadata.is_dir() {
        decode_flat_files_dir(input, output)
    } else if metadata.is_file() {
        handle_file(&PathBuf::from(input), output)
    } else {
        Err(DecodeError::InvalidInput)
    }
}

fn decode_flat_files_dir(input: &str, output: Option<&str>) -> Result<Vec<Block>, DecodeError> {
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
        match handle_file(&path.path(), output) {
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

pub fn handle_file(path: &PathBuf, output: Option<&str>) -> Result<Vec<Block>, DecodeError> {
    let mut input_file = File::open(path).map_err(DecodeError::IoError)?;
    let dbin_file = DbinFile::try_from_read(&mut input_file)?;

    if dbin_file.content_type != "ETH" {
        return Err(DecodeError::InvalidContentType(dbin_file.content_type));
    }

    let mut blocks: Vec<Block> = vec![];

    for message in dbin_file.messages {
        blocks.push(handle_block(message, output)?);
    }

    Ok(blocks)
}

pub fn decode_flat_files_iter<'a, I: Iterator<Item = &'a [u8]>>(
    iter: I,
) -> Result<Vec<Block>, DecodeError> {
    let mut blocks: Vec<Block> = vec![];

    for buf in iter {
        blocks.extend_from_slice(&handle_buf(buf)?);
    }

    Ok(blocks)
}

pub fn handle_buf(buf: &[u8]) -> Result<Vec<Block>, DecodeError> {
    let dbin_file = DbinFile::try_from_read(&mut Cursor::new(buf))?;

    let mut blocks: Vec<Block> = vec![];

    for message in dbin_file.messages {
        blocks.push(handle_block(message, None)?);
    }

    Ok(blocks)
}

fn handle_block(message: Vec<u8>, output: Option<&str>) -> Result<Block, DecodeError> {
    let message: protos::bstream::Block = Message::parse_from_bytes(&message)
        .map_err(|err| DecodeError::ProtobufError(err.to_string()))?;

    let block: Block = Message::parse_from_bytes(&message.payload_buffer)
        .map_err(|err| DecodeError::ProtobufError(err.to_string()))?;

    check_receipt_root(&block)?;
    check_transaction_root(&block)?;

    if let Some(output) = output {
        let file_name = format!("{}/block-{}.json", output, block.number);
        let mut out_file = File::create(file_name)?;

        let block_json = protobuf_json_mapping::print_to_string(&block)
            .map_err(|err| DecodeError::ProtobufError(err.to_string()))?;

        out_file
            .write_all(block_json.as_bytes())
            .map_err(DecodeError::IoError)?;
    }

    Ok(block)
}

#[cfg(test)]
mod tests {
    use crate::dbin::DbinFile;
    use crate::protos::block::Block;
    use crate::receipts::check_receipt_root;
    use crate::{handle_file, protos, receipts};
    use protobuf::Message;
    use std::fs::File;
    use std::path::PathBuf;

    #[test]
    fn test_handle_file() {
        let path = PathBuf::from("example0017686312.dbin");

        let result = handle_file(&path, None);

        assert!(result.is_ok());
    }

    #[test]
    fn test_check_valid_root_fail() {
        let path = PathBuf::from("example0017686312.dbin");
        let mut file = File::open(path).expect("Failed to open file");
        let dbin_file = DbinFile::try_from_read(&mut file).expect("Failed to parse dbin file");

        let message = dbin_file.messages[0].clone();

        let message: protos::bstream::Block =
            Message::parse_from_bytes(&message).expect("Failed to parse message");
        let mut block: Block =
            Message::parse_from_bytes(&message.payload_buffer).expect("Failed to parse block");

        block.balance_changes.pop();

        let result = check_receipt_root(&block);
        matches!(
            result,
            Err(receipts::error::ReceiptError::MismatchedRoot(_, _))
        );
    }
}
