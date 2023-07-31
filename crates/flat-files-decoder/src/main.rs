mod protos;
mod dbin;
mod receipts;


use std::any::Any;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use protobuf::Message;
use crate::dbin::DbinFile;
use crate::protos::block::Block;
use crate::receipts::check_valid_root;

fn handle_block(message: Vec<u8>) -> anyhow::Result<Block> {
    let message: protos::bstream::Block = Message::parse_from_bytes(&message)?;

    let block: Block = Message::parse_from_bytes(&message.payload_buffer)?;

    check_valid_root(&block)?;

    let file_name = format!("out/block-{}.json", block.number);
    let mut out_file = File::create(file_name)?;

    let block_json = protobuf_json_mapping::print_to_string(&block)?;

    out_file.write_all(block_json.as_bytes())?;

    Ok(block)
}

fn handle_file(path: PathBuf) -> anyhow::Result<Vec<Block>> {
    let input_file = File::open(path)
        .expect("Failed to open file");

    let dbin_file = DbinFile::from_file(input_file)
        .expect("Invalid dbin file");

    if dbin_file.content_type != "ETH" {
        panic!("Invalid content type: {}", dbin_file.content_type);
    }

    let mut blocks: Vec<Block> = vec![];

    for message in dbin_file.messages {
        blocks.push(handle_block(message)?);
    }

    Ok(blocks)
}

fn main() {
    let paths = fs::read_dir("input_files")
        .expect("Failed to read input_files directory");

    let mut blocks: Vec<Block> = vec![];
    for path in paths {
        let path = path.expect("Failed to read path");
        match path.path().extension() {
            Some(ext) => {
                if ext != "dbin" {
                    continue;
                }
            },
            None => continue
        };

        println!("Processing file: {}", path.path().display());
        match handle_file(path.path()) {
            Ok(file_blocks) => {
                blocks.extend(file_blocks);
            },
            Err(err) => {
                println!("Failed to process file: {}", err);
            }
        }
    }

    println!("Total blocks: {}", blocks.len());

}


