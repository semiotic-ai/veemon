# Verifiable Extraction Protocol Buffers in Rust

This module provides Rust implementations of StreamingFast's Firehose protocol buffer
definitions, enabling efficient encoding and decoding of data for Ethereum blockchains via StreamingFast’s framework for streaming blockchain data.

## Protobuffer definitions

### [`block.proto`](https://github.com/streamingfast/firehose-ethereum/blob/335607aac766f9f3c6946d8b1ad3c8e36ab70930/proto/sf/ethereum/type/v2/type.proto)

Representation of the tracing of a block in the Ethereum blockchain.

### [`sol_block.proto`](https://github.com/streamingfast/firehose-solana/blob/develop/proto/sf/solana/type/v1/type.proto)

Representation of the tracing of a block in the Solana blockchain.

### [`bstream.proto`](https://github.com/streamingfast/bstream/blob/develop/proto/sf/bstream/v1/bstream.proto)

`Block` type from the Streamingfast block streaming Handlers library. Lower level building block of dfuse.

## Usage

To ingest these block types from flat files, check out
[`veemon/crates/decoder`](../decoder/index.html).

For a high-level Rust client to use with Firehose endpoint providers like Pinax or StremaingFast,
check out [`semiotic-ai/firehose-client`](https://github.com/semiotic-ai/firehose-client).
