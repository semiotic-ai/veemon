// SPDX-FileCopyrightText: 2024- Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Reads and decodesencoded block files from mainnet ethereum.
//! this example depends on a .dbin file containing the blocks
//! being previolusy encoded. There is the an exmaple in the [`crates::encoder`]

use flat_files_decoder::{read_blocks_from_reader, AnyBlock, Compression};
use std::{env, error::Error, fs::File, io::BufReader};

fn decode_all(path: &str) -> Result<Vec<AnyBlock>, Box<dyn Error>> {
    if !std::path::Path::new(path).exists() {
        return Err("file not found for decoding. Run cargo --example encode_mainnet for generating files that match this example".into());
    }
    let f = File::open(path)?;
    let mut r = BufReader::new(f);
    let blocks = read_blocks_from_reader(&mut r, Compression::None)?;
    Ok(blocks)
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/mainnet_eth_blocks_12965000_5.dbin".to_string());

    let blocks = decode_all(&path)?;
    println!("Decoded {} blocks from {}", blocks.len(), &path);

    if let Some(first) = blocks.first().and_then(|b| b.as_eth_block()) {
        println!("First block number: {}", first.number);
    }
    Ok(())
}
