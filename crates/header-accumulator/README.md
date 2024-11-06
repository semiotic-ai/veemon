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
It works in conjunction with [flat-files-decoder](https://github.com/semiotic-ai/flat-files-decoder)
to offer a comprehensive solution.

## Testing

Some tests depend on [flat-files-decoder](https://github.com/semiotic-ai/flat-files-decoder) as a development dependency.

Run all tests with:

```terminal
cargo test
```
