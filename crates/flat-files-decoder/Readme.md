# Flat files decoder for firehose

## Usage

### Prerequisites
- Rust installed
- Cargo installed
- Firehose dbin files to decode
  - An example file is provided `example0017686312.dbin`

### Running
- Run `cargo run --release` in the root directory of the project
- The program will decode all files in the input_files directory
  - It will verify the receipt root & transaction root matches the computed one for all blocks

### Options
- `--input <path>`: Specify a directory or single file to read from (default: input_files)
- `--output <path>`: Specify a directory to write output files to (if missing it will not write to disk)

### Benchmarks
- Run `cargo bench` in the root directory of the project
- Benchmark results will be output to the terminal
- Benchmark time includes reading from disk & writing output to disk
- Results can be found in `target/criterion/report/index.html`
