# Header accumulator

This crate is used to accumulate block headers and compare them
against header accumulators. This process is used to verify the authenticity of these blocks


## Getting Started

### Prerequisites
- [Rust (stable)](https://www.rust-lang.org/tools/install)
- Cargo (Comes with Rust by default)
- [protoc](https://grpc.io/docs/protoc-installation/)

## Running

### Commands

- `era_validate`: Validates entire ERAs of flat files against Header Accumulators. Use this command to ensure data integrity across different ERAs.

- `generate_inclusion_proof`: Generates inclusion proofs for a range of blocks. This is useful for verifying the presence of specific blocks within a dataset.

- `verify_inclusion_proof`: Verifies inclusion proofs for a range of blocks. Use it to confirm the accuracy of inclusion proofs you have.


### Options

- `-h, --help`: Display a help message that includes usage, commands, and options.


## Goals

Our goal is to provide a tool that can be used to verify
blocks


## Testing
Some tests depend on [flat-files-decoder] to work, so it is used as a development dependency. 

### Coverage

Generate code coverage reports with `cargo llvm-cov --html` and open them with `open ./target/llvm-cov/html/index.html`. 