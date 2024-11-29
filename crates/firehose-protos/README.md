# `firehose-protos`

StreamingFast's Firehose protocol buffers compiled to Rust,
used in [header-accumulator](./../header-accumulator/Readme.md)
and [flat-files-decoder](./../flat-files-decoder/Readme.md).

## Protobuffer definitions

### [`block.proto`](https://github.com/streamingfast/firehose-ethereum/blob/335607aac766f9f3c6946d8b1ad3c8e36ab70930/proto/sf/ethereum/type/v2/type.proto)

Representation of the tracing of a block in the Ethereum blockchain.

### [`bstream.proto`](https://github.com/streamingfast/bstream/blob/develop/proto/sf/bstream/v1/bstream.proto)

`Block` type from the Streamingfast block streaming Handlers library. Lower level building block of dfuse.
