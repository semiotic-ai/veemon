mod protos;
mod dbin;

use std::fs;
use std::fs::File;
use std::io::Write;
use protobuf::Message;
use crate::dbin::DbinFile;

fn handle_block(message: Vec<u8>) {
    let message: protos::bstream::Block = Message::parse_from_bytes(&message)
        .expect("Failed to parse message");

    let block: protos::block::Block = Message::parse_from_bytes(&message.payload_buffer)
        .expect("Failed to parse block");

    let file_name = format!("out/block-{}.json", block.number);
    let mut out_file = File::create(file_name).expect("Failed to create file");

    let block_json = protobuf_json_mapping::print_to_string(&block)
        .expect("Failed to convert block to json");

    out_file.write_all(block_json.as_bytes())
        .expect("Failed to write json file");
}

fn handle_file(path: &str) {
    let input_file = File::open(path)
        .expect("Failed to open file");

    let dbin_file = DbinFile::from_file(input_file)
        .expect("Invalid dbin file");

    if dbin_file.content_type != "ETH" {
        panic!("Invalid content type: {}", dbin_file.content_type);
    }

    for message in dbin_file.messages {
        handle_block(message);
    }
}

fn main() {
    let paths = fs::read_dir("input_files")
        .expect("Failed to read input_files directory");

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

        handle_file(path.path().to_str().expect("Failed to convert path to string"));
    }

}


