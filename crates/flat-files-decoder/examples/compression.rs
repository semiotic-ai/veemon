//! Example of reading blocks from compressed and uncompressed files.
//!
//! This example demonstrates how to read blocks from compressed and uncompressed files.
//!
//! Prerequisites:
//! A dbin file both compressed and uncompressed.

use std::{fs::File, io::BufReader};

use flat_files_decoder::{read_blocks_from_reader, Compression};

fn main() {
    let path = "example.dbin.zst";
    let blocks_compressed =
        read_blocks_from_reader(create_reader(path), Compression::Zstd).unwrap();
    assert_eq!(blocks_compressed.len(), 100);

    let path = "example.dbin";
    let blocks_decompressed =
        read_blocks_from_reader(create_reader(path), Compression::None).unwrap();
    assert_eq!(blocks_compressed.len(), blocks_decompressed.len());
    for (b1, b2) in blocks_compressed.into_iter().zip(blocks_decompressed) {
        assert_eq!(b1.hash, b2.hash);
        assert_eq!(b1.number, b2.number);
        assert_eq!(b1.size, b2.size);
        assert_eq!(b1.header, b2.header);
        assert_eq!(b1.detail_level, b2.detail_level);
        assert_eq!(b1.uncles, b2.uncles);
        assert_eq!(b1.code_changes, b2.code_changes);
        assert_eq!(b1.balance_changes, b2.balance_changes);
        assert_eq!(b1.transaction_traces, b2.transaction_traces);
        assert_eq!(b1.system_calls, b2.system_calls);
    }
}

fn create_reader(path: &str) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}
