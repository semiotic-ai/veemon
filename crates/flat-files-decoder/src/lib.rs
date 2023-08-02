mod protos;
mod dbin;
mod receipts;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use anyhow::anyhow;
use protobuf::Message;
use dbin::DbinFile;
use protos::block::Block;
use receipts::check_valid_root;

pub fn decode_flat_files(dir: &str) -> anyhow::Result<Vec<Block>> {
    let paths = fs::read_dir(dir)?;

    let mut blocks: Vec<Block> = vec![];
    for path in paths {
        let path = path?;
        match path.path().extension() {
            Some(ext) => {
                if ext != "dbin" {
                    continue;
                }
            },
            None => continue
        };

        println!("Processing file: {}", path.path().display());
        match handle_file(&path.path()) {
            Ok(file_blocks) => {
                blocks.extend(file_blocks);
            },
            Err(err) => {
                println!("Failed to process file: {}", err);
            }
        }
    };

    Ok(blocks)
}

pub fn handle_file(path: &PathBuf) -> anyhow::Result<Vec<Block>> {
    let input_file = File::open(path)?;

    let dbin_file = DbinFile::try_from(input_file)?;

    if dbin_file.content_type != "ETH" {
        return Err(anyhow!("Invalid content type: {}", dbin_file.content_type));
    }

    let mut blocks: Vec<Block> = vec![];

    for message in dbin_file.messages {
        blocks.push(handle_block(message)?);
    }

    Ok(blocks)
}

fn handle_block(message: Vec<u8>) -> anyhow::Result<Block> {
    let message: protos::bstream::Block = Message::parse_from_bytes(&message)?;

    let block: Block = Message::parse_from_bytes(&message.payload_buffer)?;

    check_valid_root(&block)?;

    // let file_name = format!("output_files/block-{}.json", block.number);
    // let mut out_file = File::create(file_name)?;
    //
    // let block_json = protobuf_json_mapping::print_to_string(&block)?;
    //
    // out_file.write_all(block_json.as_bytes())?;

    Ok(block)
}

#[test]
fn test_handle_file() {
    let path = PathBuf::from("example0017686312.dbin");

    let result = handle_file(&path);

    assert!(result.is_ok());
}

#[test]
fn test_check_valid_root_fail() {
    let path = PathBuf::from("example0017686312.dbin");
    let file = File::open(path).expect("Failed to open file");
    let dbin_file = DbinFile::try_from(file)
        .expect("Failed to parse dbin file");

    let message = dbin_file.messages[0].clone();

    let message: protos::bstream::Block = Message::parse_from_bytes(&message)
        .expect("Failed to parse message");
    let mut block: Block = Message::parse_from_bytes(&message.payload_buffer)
        .expect("Failed to parse block");

    block.balance_changes.pop();

    let result = check_valid_root(&block);
    matches!(result, Err(receipts::error::InvalidReceiptError::ReceiptRoot(_, _)));
}



