# Encoder Crate

DBIN-aligned encoder for producing streams consumable by the decoder in `crates/decoder`.

Purpose:
- Provide encoding utilities to convert raw blockchain data into a DBIN-like binary stream suitable for storage or transport and consumption by the decoder.

Usage:
- Add as a dependency in your project, then call the encoder methods on byte slices or SSZ payloads.

Examples in this repository:
- See the usage in the encoder examples:
  - `crates/encoder/examples/encode_beacon.rs` (uses `Encoder::new_v1("BEA")` with `encode_blocks`).
  - `crates/encoder/examples/encode_mainnet.rs` (uses `Encoder::new_v1("ETH")` with `encode_blocks`).
  - `crates/encoder/examples/encode_state.rs` (uses `Encoder::new_v1("STA")` with `encode_value`).

Build & test:
- `cargo build -p encoder` or `cargo test -p encoder -- --nocapture`.
