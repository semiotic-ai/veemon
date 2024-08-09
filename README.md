# Forrestrie

## Overview

Validating post-merge Ethereum data.

## Documentation

- Notion doc on
[Post-merge Header Record Data Structure](https://www.notion.so/semiotic/Post-merge-header_record-data-structure-7290d03d356946188bdb9ac29366f510?pvs=4).
- [Beacon Chain `BeaconState` spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/capella/beacon-chain.md#beaconstate)
- [Beacon Chain `BeaconBlockBody` spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/deneb/beacon-chain.md#beaconblockbody)
- The [fork of `sigp/lighthouse`](https://github.com/semiotic-ai/lighthouse) we've been spiking.
- [Google Drive shared resources](https://drive.google.com/drive/u/1/folders/15diM-Gu4WFg9FrMWti3_B8xP0J0szUhW),
including `head-state.json` used in `beacon_state.rs` tests.

## Prerequisites

> [!NOTE]
> You need to add the `head-state.json` file from our shared Google Drive to
> the root of this repo to run tests, as well as the
> [`bb-8786333.json`](https://drive.google.com/file/d/1-9SgmdxrOU5t1XlBc0hsRcEM-xZVN91N/view?usp=drive_link)!
