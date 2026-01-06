// SPDX-FileCopyrightText: 2024- Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0
//! # Fetch Beacon State
//!
//! Demonstrates how to fetch the head beacon state from a light client data endpoint
//! and encode it to a DBIN stream written to the filesystem (similar to the beacon
//! and ETH block examples).

use flat_files_encoder::Encoder;
use types::{BeaconState, MainnetEthSpec};

use ssz::Encode;
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

#[derive(Deserialize)]
struct DebugStateResponse {
    data: BeaconState<MainnetEthSpec>,
}

#[tokio::main]
async fn main() {
    const LIGHT_CLIENT_DATA_URL: &str =
        "https://docs-demo.quiknode.pro/eth/v2/debug/beacon/states/head";

    let resp = reqwest::get(LIGHT_CLIENT_DATA_URL).await.unwrap();
    let DebugStateResponse { data: state } = resp.json::<DebugStateResponse>().await.unwrap();
    // Serialize as SSZ bytes
    let payload = state.as_ssz_bytes();

    // Encode and write to /tmp
    let encoder = Encoder::new_v1("STA");

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let path = format!("/tmp/mainnet_beacon_state_head_{}.dbin", ts);
    let mut f = File::create(&path).unwrap();
    encoder
        .encode_bytes_to_writer(&mut f, std::iter::once(payload.as_slice()))
        .expect("DBIN encoding failed");

    println!("Wrote {}", path);
}
