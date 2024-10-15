# Veemon

## Overview

Verifying Ethereum data.

## Crates

### Forrestrie

Library of types and methods for verifying post-merge Ethereum data.

### Firehose Client

Support for interfacing programmatically with Firehose gRPC endpoints.

For more information see the
[`firehose-client/README`](./crates/firehose-client/README.md).

## Documentation

- Notion doc on
[Post-merge Header Record Data Structure](https://www.notion.so/semiotic/Post-merge-header_record-data-structure-7290d03d356946188bdb9ac29366f510?pvs=4).
- [Beacon Chain `BeaconState` spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/capella/beacon-chain.md#beaconstate)
- [Beacon Chain `BeaconBlockBody` spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/deneb/beacon-chain.md#beaconblockbody)
- The [fork of `sigp/lighthouse`](https://github.com/semiotic-ai/lighthouse) we've been spiking.
- [Google Drive shared resources](https://drive.google.com/drive/folders/19QBMHZFAV7uo_Cu4RwLPTwGpBcQMd-hy),
including `head-state.json` used in `beacon_state.rs` tests.

## Examples

Here's an example of how to run one of the examples in the `forrestrie` crate:

```terminal
cd crates/forrestrie && cargo run -- --examples historical_state_roots_proof
```

Use environment variables to provide Firehose Ethereum and Firehose Beacon providers of
your choice.

To do so, place a `.env` file in the root of the crate you want to run examples for. 
Your `.env` file should look like something this, depending on your requirements:

```shell
FIREHOSE_ETHEREUM_URL=<YOUR-FIREHOSE-ETHEREUM-URL>
FIREHOSE_ETHEREUM_PORT=<YOUR-FIREHOSE-ETHEREUM-PORT>
FIREHOSE_BEACON_URL=<YOUR-FIREHOSE-BEACON-URL>
FIREHOSE_BEACON_PORT=<YOUR-FIREHOSE-BEACON-PORT>
BEACON_API_KEY=<YOUR-API-KEY>
ETHEREUM_API_KEY=<YOUR-API-KEY>
```
