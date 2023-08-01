# Flat files decoder for firehose

## Usage

### Prerequisites
- Rust installed
- Cargo installed
- Firehose dbin files to decode
  - An example file is provided `example0017686312.dbin`
  - Input files must be in the directory input_files
  - Decoded json files will be output to the directory output_files

### Running
- Run `cargo run --release` in the root directory of the project
- The program will decode all files in the input_files directory
  - In doing so it will verify the receipt root matches the real one for all blocks

### Benchmarks
- Run `cargo bench` in the root directory of the project
- Benchmark results will be output to the terminal
- Benchmark time includes reading from disk & writing output to disk
- Results can be found in `target/criterion/report/index.html`
