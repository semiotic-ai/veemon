# Encoder Crate

Purpose:
- Provide encoding utilities to convert raw block data into a binary representation for storage or transport.

Current status:
- Skeleton. Exposes an `encode` function as a placeholder for a real encoding algorithm.

Public API (stable-ish):
- `pub fn encode(input: &[u8]) -> Vec<u8>`: Encode data into a binary blob.

Usage:
- Add as a dependency in your project, then call `encoder::encode` on byte slices.

Build & test:
- `cargo build -p encoder` or `cargo test -p encoder -- --nocapture`.
