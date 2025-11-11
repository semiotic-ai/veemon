use flat_files_decoder::{read_blocks_from_reader, AnyBlock, Compression};
use std::{env, error::Error, fs::File, io::BufReader};

fn decode_all(path: &str) -> Result<Vec<AnyBlock>, Box<dyn Error>> {
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
