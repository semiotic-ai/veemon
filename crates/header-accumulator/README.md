# Header Accumulator

`header_accumulator` is a Rust library used to accumulate and verify
block headers by comparing them against header accumulators, helping
to ensure the authenticity of blockchain data. This crate is designed
primarily for use as a library, requiring parsed blocks as input.

## Overview

Check out the crate documentation in your browser by running, from
the root of the `veemon` workspace:

```terminal
cd crates/header-accumulator && cargo doc --open
```

## Getting Started

### Prerequisites

- [Rust (stable)](https://www.rust-lang.org/tools/install)
- Cargo (included with Rust by default)
- [protoc](https://grpc.io/docs/protoc-installation/)

## Features

- **`era_validate`**: Validates entire ERAs of flat files against
  Header Accumulators. Use this command to ensure data integrity across
  multiple ERAs.
- **`generate_inclusion_proof`**: Generates inclusion proofs for a
  specified range of blocks, useful for verifying the presence of
  specific blocks within a dataset.
- **`verify_inclusion_proof`**: Verifies inclusion proofs for a 
  specified range of blocks. Use this to confirm the accuracy of
  inclusion proofs.

### Options

- `-h, --help`: Displays a help message that includes usage 
  information, commands, and options.

## Goals

The main objective of this library is to provide a tool for verifying
blocks from [StreamingFast Firehose](https://firehose.streamingfast.io/).
It works in conjunction with [decoder](https://github.com/semiotic-ai/decoder)
to offer a comprehensive solution.

## Testing

Some tests depend on [decoder](../decoder/README.md) as a development dependency.

Run all tests with:

```terminal
cargo test
```

## Examples

### Inclusion proof

```rust,no_run
use std::{fs::File, io::BufReader};
use flat_files_decoder::{read_blocks_from_reader, Compression};
use header_accumulator::{
    generate_inclusion_proofs, verify_inclusion_proofs, Epoch, EraValidateError, ExtHeaderRecord,
};

fn main() -> Result<(), EraValidateError> {
   let mut headers: Vec<ExtHeaderRecord> = Vec::new();

    for flat_file_number in (0..=8200).step_by(100) {
        let file = format!(
            "your_files/ethereum_firehose_first_8200/{:010}.dbin",
            flat_file_number
        );
        match read_blocks_from_reader(
            BufReader::new(File::open(&file).unwrap()),
            Compression::None,
        ) {
            Ok(blocks) => {
                headers.extend(
                    blocks
                        .iter()
                        .map(|block| ExtHeaderRecord::try_from(block).unwrap())
                        .collect::<Vec<ExtHeaderRecord>>(),
                );
            }
            Err(e) => {
                eprintln!("error: {:?}", e);
                break;
            }
        }
    }

    let start_block = 301;
    let end_block = 402;
    let headers_to_prove: Vec<_> = headers[start_block..end_block]
        .iter()
        .map(|ext| ext.full_header.as_ref().unwrap().clone())
        .collect();
    let epoch: Epoch = headers.try_into().unwrap();

    let inclusion_proof = generate_inclusion_proofs(vec![epoch], headers_to_prove.clone())
        .unwrap_or_else(|e| {
            println!("Error occurred: {}", e);
            std::process::exit(1);
        });
    assert_eq!(inclusion_proof.len(), headers_to_prove.len());

    let proof_headers = headers_to_prove
        .into_iter()
        .zip(inclusion_proof)
        .map(|(header, proof)| proof.with_header(header))
        .collect::<Result<Vec<_>, _>>()?;

    // Verify inclusion proof
    assert!(verify_inclusion_proofs(None, proof_headers).is_ok());

    println!("Inclusion proof verified successfully!");

    Ok(())
}    
```

### Era validator

```rust,no_run
use std::{fs::File, io::BufReader};

use flat_files_decoder::{read_blocks_from_reader, Compression};
use header_accumulator::{Epoch, EraValidateError, EraValidator, ExtHeaderRecord};
use tree_hash::Hash256;

fn create_test_reader(path: &str) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}

fn main() -> Result<(), EraValidateError> {
    let mut headers: Vec<ExtHeaderRecord> = Vec::new();
    for number in (0..=8200).step_by(100) {
        let file_name = format!(
            "your-test-assets/ethereum_firehose_first_8200/{:010}.dbin",
            number
        );
        let reader = create_test_reader(&file_name);
        let blocks = read_blocks_from_reader(reader, Compression::None).unwrap();
        let successful_headers = blocks
            .iter()
            .cloned()
            .map(|block| ExtHeaderRecord::try_from(&block))
            .collect::<Result<Vec<_>, _>>()?;
        headers.extend(successful_headers);
    }
    assert_eq!(headers.len(), 8300);
    assert_eq!(headers[0].block_number, 0);
    let era_verifier = EraValidator::default();
    let epoch: Epoch = headers.try_into().unwrap();
    let result = era_verifier.validate_era(&epoch)?;
    let expected = Hash256::new([
        94, 193, 255, 184, 195, 177, 70, 244, 38, 6, 199, 76, 237, 151, 61, 193, 110, 197, 161, 7,
        192, 52, 88, 88, 195, 67, 252, 148, 120, 11, 66, 24,
    ]);
    assert_eq!(result, expected);

    println!("Era validated successfully!");

    Ok(())
}
```