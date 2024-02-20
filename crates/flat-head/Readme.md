# Flat head

Flat head is developed to be a crate that verifies flat files generated from Firehose against header accumulators.


## Getting Started

### Prerequisites
- [Rust (stable)](https://www.rust-lang.org/tools/install)
- Cargo (Comes with Rust by default)
- [protoc](https://grpc.io/docs/protoc-installation/)
- Firehose dbin files to decode

## Running

### Commands


### Options



## Usage Examples

Here are some examples of how to use the commands:

1.  To validate era files in a folder, a start epoch and a end epoch must be provided. `-d` flag can be used for debugging or log information.
```
cargo run --  -d  era-validate --input  ./tests/ethereum_firehose_first_8200/ --start-epoch 0 --end-epoch 1
```

## Goals

Our goal is to provide The Graph's Indexers the tools to trustlessly share flat files with cryptographic guarantees 
that the data in the flat files is part of the canonical history of the Ethereum blockchain, 
enabling Indexers to quickly sync all historical data and begin serving data with minimal effort.
