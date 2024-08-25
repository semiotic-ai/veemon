# sf-protos

StreamingFast's Rust-compiled protocol buffers, used in [header-accumulator](https://github.com/semiotic-ai/header_accumulator)
and [flat-files-decoder](https://github.com/semiotic-ai/flat-files-decoder)

## Protobuffer definitions

- `block.proto`: <https://github.com/streamingfast/firehose-ethereum/blob/335607aac766f9f3c6946d8b1ad3c8e36ab70930/proto/sf/ethereum/type/v2/type.proto>

## Tests

### Data

`exec_layer_block_20562650_header.json` was serialized to JSON from a
`sf_protos::ethereum::r#type::v2::Block` received from a Firehose endpoint
over gRPC and then redacted to only contain data for the fields necessary
to compute the `Header`.
