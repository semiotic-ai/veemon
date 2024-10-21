# sf-protos

StreamingFast's and Pinax's Rust-compiled protocol buffers for Firehose,
used in [header-accumulator](https://github.com/semiotic-ai/header_accumulator),
[flat-files-decoder](https://github.com/semiotic-ai/flat-files-decoder), and
[forrestrie](https://github.com/semiotic-ai/forrestrie).

## Protobuffer definitions

- `block.proto`: <https://github.com/streamingfast/firehose-ethereum/blob/335607aac766f9f3c6946d8b1ad3c8e36ab70930/proto/sf/ethereum/type/v2/type.proto>
- `firehose.proto`: <https://github.com/streamingfast/proto/blob/develop/sf/firehose/v2/firehose.proto>
- `type.proto`: <https://github.com/pinax-network/firehose-beacon/blob/main/proto/sf/beacon/type/v1/type.proto>

## Tests

### Data

`exec_layer_block_20562650_header.json` was serialized to JSON from a
`sf_protos::ethereum::r#type::v2::Block` received from a Firehose endpoint
over gRPC and then redacted to only contain data for the fields necessary
to compute the `Header`.
