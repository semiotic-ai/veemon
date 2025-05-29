// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Example of reading blocks from compressed and uncompressed files.
//!
//! This example demonstrates how to read blocks from compressed and uncompressed files.
//!
//! Prerequisites:
//! A v0 dbin file both compressed and uncompressed, with message data representing
//! Ethereum Blocks.

use std::{fs::File, io::BufReader};

use flat_files_decoder::{read_blocks_from_reader, AnyBlock, Compression};

fn main() {
    let path = "example.dbin.zst";
    let blocks_compressed =
        read_blocks_from_reader(create_reader(path), Compression::Zstd).unwrap();
    let mut block = blocks_compressed.first().unwrap();
    assert!(block.is_eth_block());
    assert_eq!(blocks_compressed.len(), 100);

    let path = "example.dbin";
    let blocks_decompressed =
        read_blocks_from_reader(create_reader(path), Compression::None).unwrap();
    block = blocks_decompressed.first().unwrap();
    assert!(block.is_eth_block());
    assert_eq!(blocks_compressed.len(), blocks_decompressed.len());
    for (b1, b2) in blocks_compressed.into_iter().zip(blocks_decompressed) {
        let v1 = b1.try_into_eth_block().unwrap();
        let v2 = b2.try_into_eth_block().unwrap();
        assert_eq!(v1.hash, v2.hash);
        assert_eq!(v1.number, v2.number);
        assert_eq!(v1.size, v2.size);
        assert_eq!(v1.header, v2.header);
        assert_eq!(v1.detail_level, v2.detail_level);
        assert_eq!(v1.uncles, v2.uncles);
        assert_eq!(v1.code_changes, v2.code_changes);
        assert_eq!(v1.balance_changes, v2.balance_changes);
        assert_eq!(v1.transaction_traces, v2.transaction_traces);
        assert_eq!(v1.system_calls, v2.system_calls);
    }
}

fn create_reader(path: &str) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}
