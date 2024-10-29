# `firehose-protos`

StreamingFast's Firehose protocol buffers compiled to Rust,
used in [header-accumulator](./../header-accumulator/Readme.md)
and [flat-files-decoder](./../flat-files-decoder/Readme.md).

## Protobuffer definitions

### [`block.proto`](https://github.com/streamingfast/firehose-ethereum/blob/335607aac766f9f3c6946d8b1ad3c8e36ab70930/proto/sf/ethereum/type/v2/type.proto)

Representation of the tracing of a block in the Ethereum blockchain.

### [`bstream.proto`](https://github.com/streamingfast/bstream/blob/develop/proto/sf/bstream/v1/bstream.proto)

`Block` type from the Streamingfast block streaming Handlers library. Lower level building block of dfuse.

### [`firehose.proto`](https://github.com/streamingfast/proto/blob/develop/sf/firehose/v2/firehose.proto)

Firehose fetch and streaming, client and server gRPC implementation.

## Examples

Here's an example of how to run one of the examples:

```terminal
cd crates/firehose-protos && cargo run -- --examples receipt_root
```

Use environment variables to provide Firehose Ethereum and Firehose
Beacon providers of your choice.

To do this, place a `.env` file in the root of `veemon`. See the
`.env.example` file in the root of this repository for what you'll need,
depending on your requirements.
