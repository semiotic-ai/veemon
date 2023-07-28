mod protos;
mod dbin;

use std::fs::File;
use std::io::Write;
use protobuf::Message;
use crate::dbin::DbinFile;

fn hanlde_message(message: Vec<u8>) {
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

fn main() {
    let input_file = File::open("example0017686312.dbin")
        .expect("Failed to open file");

    let dbin_file = DbinFile::from_file(input_file)
        .expect("Invalid dbin file");

    if dbin_file.content_type != "ETH" {
        panic!("Invalid content type: {}", dbin_file.content_type);
    }

    for message in dbin_file.messages {
        hanlde_message(message);
    }
}


