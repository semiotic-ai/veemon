// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use flat_files_decoder::{stream_blocks, AnyBlock, EndBlock, Reader};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Cursor, Write},
    path::PathBuf,
};

fn main() {
    let mut buffer = Vec::new();
    let cursor: Cursor<&mut Vec<u8>> = Cursor::new(&mut buffer);

    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // build absolute paths to your test files, relative to the crate root
    let inputs: Vec<PathBuf> = ["0000000000.dbin"]
        .into_iter()
        .map(|f| crate_dir.join("tests").join(f))
        .collect();
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

    let blocks: Vec<AnyBlock> = stream_blocks(Reader::Buf(reader), EndBlock::Block(99))
        .unwrap()
        .collect();

    assert_eq!(blocks.len(), 100);
    println!("read_blocks.rs done");

    println!("Read blocks:");
    for block in blocks {
        // `block` is an `AnyBlock`; convert into an `EthBlock`.
        // If test file does not contain `EthBlock`s, this will produce a ConversionError.
        let eth_block = block.try_into_eth_block().unwrap();
        println!("{:?}", eth_block.number);
    }
}
