# Flat files decoder for firehose

[![CI status](https://github.com/semiotic-ai/flat-files-decoder/workflows/ci/badge.svg)][gh-ci]

this crate is designed to decompress and decode headers from [binary files, which are called flat files,](https://github.com/streamingfast/firehose-ethereum/blob/develop/proto/sf/ethereum/type/v2/type.proto) generated from Firehose. Flat files store all information necessary to reconstruct the transaction and receipt tries. It also checks the validity of receipt roots and transaction roots present in the block headers by recalculating them via the block body data. Details of the implementation can be found [here](https://github.com/streamingfast/dbin?tab=readme-ov-file).
This check ensures that receipt logs and transaction data stored in the flat files are internally consistent with the block headers also stored in the flat files.

This tool was first presented as a mean to enhance the performance and verifiability of The Graph protocol. However,
it turns out it could be used as a solution for EIP-4444 problem of full nodes stopping to provide historical data over one year.
The idea is that the flat files that this crate can decode could also be used as an archival format similar to era1 files if the
transaction and receipt data stored in the flat files can be verified to be part of the Ethereum canonical history. 

For more information, read our ethresear.ch [here](https://ethresear.ch/t/using-the-graph-to-preserve-historical-data-and-enable-eip-4444/17318) and our pre-merge
solution implementation [here](https://github.com/semiotic-ai/header_accumulator) to see how we can verify that the flat files are consistent with Ethereum's canonical chain. 

## Getting Started

### Prerequisites
- [Rust (stable)](https://www.rust-lang.org/tools/install)
- Cargo (Comes with Rust by default)
- [protoc](https://grpc.io/docs/protoc-installation/)
- Firehose dbin files to decode
  - An example file is provided `example0017686312.dbin`

## Running

### Commands

The tool provides the following commands for various operations:

- `stream`: Stream data continuously.
- `decode`: Decode files from input to output.
- `help`: Print this message or the help of the given subcommand(s).

### Options

You can use the following options with the commands for additional functionalities:

- `-h, --help`: Print help information about specific command and options.
- `-V, --version`: Print the version information of the tool.


#### NOTICE: either streaming or reading from directory it will verify the receipt root & transaction root matches the computed one for all blocks

## Usage Examples

Here are some examples of how to use the commands:

1. To stream data continuously from `stdin`:

  ```bash
  # simply turning on stream stdin reading
  cargo run stream
  
  # or from files into stdin
  cat example0017686312.dbin | cargo run stream
  ```

This will output decoded header records as bytes into `stdout`

2. To check a folder of dbin files:

```bash
cargo run decode --input ./input_files/
```

This will store the block headers as json format in the output folder. 
By passing `--headers-dir` a folder of assumed valid block headers can be provided to compare
with the input flat files. Valid headers can be pulled from the [sync committee subprotocol](https://github.com/ethereum/annotated-spec/blob/master/altair/sync-protocol.md) for post-merge data.

**NOTICE:**For pre-merge data another approach using the [header accumulator](https://github.com/ethereum/portal-network-specs/blob/8ad5bc33cb0d4485d2eab73bf2decc43e7566a8f/history-network.md#the-header-accumulator) is necessary since
sync committees will not provide these headers.

## Goals

Our goal is to provide The Graph's Indexers the tools to trustlessly share flat files with cryptographic guarantees 
that the data in the flat files is part of the canonical history of the Ethereum blockchain, 
enabling Indexers to quickly sync all historical data and begin serving data with minimal effort.

## Benchmarking
- Run `cargo bench` in the root directory of the project
- Benchmark results will be output to the terminal
- Benchmark time includes reading from disk & writing output to disk
- Results can be found in `target/criterion/report/index.html`

For proper benchmarking of future improvements, fixes and features please compare baselines.
Refer to [the end of this section of Criterion documentation](https://bheisler.github.io/criterion.rs/book/user_guide/command_line_options.html) for more information on creating and comparing baselines.