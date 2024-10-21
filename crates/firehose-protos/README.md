# `firehose-protos`

StreamingFast's and Pinax's Rust-compiled protocol buffers for Firehose,
used in [header-accumulator](./../header-accumulator/Readme.md),
[flat-files-decoder](./../flat-files-decoder/Readme.md), and
[forrestrie](./../../README.md).

## Protobuffer definitions

### [`block.proto`](https://github.com/streamingfast/firehose-ethereum/blob/335607aac766f9f3c6946d8b1ad3c8e36ab70930/proto/sf/ethereum/type/v2/type.proto)

Representation of the tracing of a block in the Ethereum blockchain.

### [`bstream.proto`](https://github.com/streamingfast/bstream/blob/develop/proto/sf/bstream/v1/bstream.proto)

`Block` type from the Streamingfast block streaming Handlers library. Lower level building block of dfuse.

### [`firehose.proto`](https://github.com/streamingfast/proto/blob/develop/sf/firehose/v2/firehose.proto)

Firehose fetch and streaming, client and server gRPC implementation.

### [`type.proto`](https://github.com/pinax-network/firehose-beacon/blob/main/proto/sf/beacon/type/v1/type.proto)

Pinax's Firehose Beacon `Block` implementation.

## Tests

### Data

`exec_layer_block_20562650_header.json` was serialized to JSON from a
`firehose_protos::ethereum::r#type::v2::Block` received from a Firehose endpoint
over gRPC and then redacted to only contain data for the fields necessary
to compute the `Header`.