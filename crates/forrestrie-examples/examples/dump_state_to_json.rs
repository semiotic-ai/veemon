/*
Copyright 2024-, Semiotic AI, Inc.
SPDX-License-Identifier: Apache-2.0
*/

//! Dumps a single head state into json. Useful for not relying on fetching it,
//! decreases the time necessary for verifying a proof.
use std::{fs::File, io::Write};

use forrestrie::beacon_state::HeadState;
use types::MainnetEthSpec;

const LIGHT_CLIENT_DATA_URL: &str =
    "https://www.lightclientdata.org/eth/v2/debug/beacon/states/head";

#[tokio::main]
async fn main() {
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Get the head state of the Beacon chain from a Beacon API provider.
    let state_handle = tokio::spawn(async move {
        let url = LIGHT_CLIENT_DATA_URL.to_string();
        println!("Requesting head state ... (this can take a while!)");
        let response = reqwest::get(url).await.unwrap();
        let head_state: HeadState<MainnetEthSpec> = response.json().await.unwrap();
        head_state
    });

    let mut file = File::create("head_state.json").expect("Failed to create file");
    let head_state = state_handle.await.unwrap();
    let json_string =
        serde_json::to_string(&head_state).expect("Failed to serialize head_state to JSON");
    file.write_all(json_string.as_bytes())
        .expect("Failed to write to file");
}
