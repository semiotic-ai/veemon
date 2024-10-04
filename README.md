# Forrestrie

## Overview

Validating post-merge Ethereum data.

### Crates

#### Forrestrie

Library of types and methods for verifying post-merge Ethereum data.

#### Firehose Client

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

## Prerequisites

> [!NOTE]
> You need to add the test assets files from our shared Google Drive to
> the root of this repo to run tests. See **Google Drive shared resources**
> above.
