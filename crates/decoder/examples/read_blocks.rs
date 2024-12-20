// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::File,
    io::{BufReader, BufWriter, Cursor, Write},
};

use firehose_protos::EthBlock as Block;
use flat_files_decoder::{stream_blocks, EndBlock, Reader};

fn main() {
    let mut buffer = Vec::new();
    let cursor: Cursor<&mut Vec<u8>> = Cursor::new(&mut buffer);
    let inputs = vec![format!("example-1.dbin"), format!("example-2.dbin")];
    {
        let mut writer = BufWriter::new(cursor);
        for i in inputs {
            let mut input = File::open(i).expect("Make sure you have some test assets!");

            std::io::copy(&mut input, &mut writer).unwrap();
            writer.flush().unwrap();
        }
    }
    let mut cursor = Cursor::new(buffer);
    cursor.set_position(0);

    let reader = BufReader::new(cursor);

    let blocks: Vec<Block> = stream_blocks(Reader::Buf(reader), EndBlock::MergeBlock)
        .unwrap()
        .collect();

    assert_eq!(blocks.len(), 2);
    println!("read_blocks.rs done");

    println!("Read blocks:");
    for block in blocks {
        println!("{:?}", block.number);
    }
}
