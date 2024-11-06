# Flat Files Decoder

## Overview

Check out the crate documentation in your browser by running, from
the root of the `veemon` workspace:

```terminal
cd crates/flat-files-decoder && cargo doc --open
```

## Running CLI Example

### Commands

The tool provides the following commands for various operations:

- `stream`: Stream data continuously.
- `decode`: Decode files from input to output.
- `help`: Print this message or the help of the given subcommand(s).

### Options

You can use the following options with the commands for additional functionalities:

- `-h, --help`: Print help information about specific command and options.
- `-V, --version`: Print the version information of the tool.

### Usage Examples

Here are some examples of how to use the commands:

1. To stream data continuously from `stdin`:

```terminal
cargo run -p flat-files-decoder --example cli stream
```

```terminal
cat example0017686312.dbin | cargo run -p flat-files-decoder --example cli stream
```

This will output decoded header records as bytes into `stdout`

1. To check a folder of dbin files:

```terminal
cargo run -p flat-files-decoder --example cli decode --input ./input_files/ --compression true
```

So, if using test data from a `test-assets/` folder in the root of the `veemon` repo:

```terminal
cargo run -p flat-files-decoder --example cli decode --input test-assets/benchmark_files/pre_merge
```

This will store the block headers as json format in the output folder. 
By passing `--headers-dir` a folder of assumed valid block headers can be provided to compare
with the input flat files. Valid headers can be pulled from the [sync committee subprotocol](https://github.com/ethereum/annotated-spec/blob/master/altair/sync-protocol.md) for post-merge data.

## Benchmarking

- Run `cargo bench` in the root directory of the project
- Benchmark results will be output to the terminal
- Benchmark time includes reading from disk & writing output to disk
- Results can be found in `target/criterion/report/index.html`

For proper benchmarking of future improvements, fixes and features please compare baselines.
Refer to [the end of this section of Criterion documentation](https://bheisler.github.io/criterion.rs/book/user_guide/command_line_options.html) for more information on creating and comparing baselines.
