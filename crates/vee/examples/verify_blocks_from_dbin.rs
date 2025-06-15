// Copyright 2025-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Example docs
//! In this example, we verify a block derived from a StreamingFast .dbin file.
//! We are verifying block number 99 from pre-Merge Ethereum as an example.

use std::{fs::File, io::BufReader};
use tree_hash::Hash256;
use vee::{read_blocks_from_reader, AnyBlock, Compression, Epoch, EraValidator, ExtHeaderRecord};

fn main() {
    // Path to .dbin file containing block to verify. This is our "extracted"
    // data. Suppose we need some information contained in a specific block
    // which we need to verify is correct.
    // First we parse the file into blocks.
    // These example dbin files contain 100 blocks each. Here we are using
    // the last block in our example file 0000000000.dbin.
    let path = "../ve-assets/dbin/pre-merge/0000000000.dbin";
    let buf_reader = BufReader::new(File::open(path).unwrap());
    let blocks = read_blocks_from_reader(buf_reader, Compression::None).unwrap();
    let any_block = blocks.last().unwrap();
    let block = any_block.clone().try_into_eth_block().unwrap();

    // `read_blocks_from_reader` includes a check that the contents of the
    // blocks in the file (receipts, transactions, and blockhash) are valid.
    // If the file was decoded successfully, the contents were verified as
    // part of that process. Here, we perform the verification again separately
    // to demonstrate what is happening.

    // `receipt_root_is_verified` reconstructs the receipts Merkle trie using the receipts
    // recorded in the block. The root of the trie is then compared against the receipts root hash
    // recorded in the block header. If they match, then the receipt contents of the
    // block are consistent with the commitment recorded in the block header.
    assert!(block.receipt_root_is_verified());
    // `transaction_root_is_verified` reconstructs the transactions Merkle trie using the transaction traces
    // recorded in the block. The root of the trie is then compared against the transactions
    // root hash recorded in the block header. If they match, then the transaction contents of the
    // block are consistent with the commitment recorded in the block header.
    assert!(block.transaction_root_is_verified());
    // `block_hash_is_verified` calculates the block hash using information from the block header,
    // including the receipts root and the transactions root. The calculated block hash is then
    // compared against the block hash recorded in the block header. If they match, then
    // the contents of the block header incorporated in the hash are consistent with
    // the hash recorded in the header. The block hash can be used to prove inclusion
    // of the block in the chain's history.
    assert!(block.block_hash_is_verified());
    println!("Block contents validated successfully.");

    // Next, show that the block (represented by its block hash) was included in
    // the chain's history, i.e., the block is valid compared against the
    // Ethereum ledger.
    // Our trusted source here is the `PreMergeAccumulator` maintained by
    // Ethereum Portal Network. For each era (8192 blocks) in the pre-Merge
    // Ethereum chain, a Merkle tree is constructed with the block hashes of
    // the era at the leaves. The root hash of the tree is recorded
    // in the accumulator. We can reconstruct the tree for a given era and
    // check the calculated root against the recorded value.

    // To reconstruct the tree for the era containing the block in question,
    // we need all block hashes from the era. For this example, we obtain
    // the blocks from more StreamingFast dbin files extracted using Firehose.
    let mut headers: Vec<ExtHeaderRecord> = Vec::new();
    for number in (0..=8200).step_by(100) {
        let file_name = format!("../ve-assets/dbin/pre-merge/{:010}.dbin", number);
        let reader = BufReader::new(File::open(file_name.clone()).unwrap());
        let blocks = read_blocks_from_reader(reader, Compression::None).unwrap();
        let successful_headers = blocks
            .iter()
            .filter_map(|block| {
                if let AnyBlock::Evm(eth_block) = block {
                    ExtHeaderRecord::try_from(eth_block).ok()
                } else {
                    // Note, Header Accumulators are currently only supported
                    // for Ethereum type blocks.
                    println!("File contains non-Ethereum Block: {}", file_name);
                    None
                }
            })
            .collect::<Vec<_>>();
        headers.extend(successful_headers);
    }
    // Arrange block headers in order and determine the era number in which they belong.
    let era: Epoch = headers.try_into().unwrap();
    // `EraValidator` struct containing a historical record of Merkle tree roots
    // from the trusted `PreMergeAccumulator`.
    let era_verifier = EraValidator::default();
    // `validate_era` calculates the root of the Merkle tree with this era's
    // block hashes at the leaves. The method includes a check that the calculated
    // value matches that in the `EraValidator` for this era.
    let result = era_verifier.validate_era(&era).unwrap();
    // Here, we compare the historical record with the calculated value separately
    // to show what is happening.
    let expected = Hash256::new([
        94, 193, 255, 184, 195, 177, 70, 244, 38, 6, 199, 76, 237, 151, 61, 193, 110, 197, 161, 7,
        192, 52, 88, 88, 195, 67, 252, 148, 120, 11, 66, 24,
    ]);
    assert_eq!(result, expected);
    println!("Extracted blocks match historical record.")
}
