// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use prost_build::Config;
use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Configure prost to derive serde traits on specific types
    let mut config = Config::new();
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.type_attribute(".", "#[allow(clippy::enum_variant_names)]");
    config.type_attribute(".", "#[allow(missing_docs)]");

    // Map Google protobuf types to prost_wkt_types
    config.extern_path(".google.protobuf.Any", "::prost_wkt_types::Any");
    config.extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp");

    tonic_build::configure()
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("descriptors.bin"))
        .compile_protos_with_config(
            config,
            &[
                "protos/block.proto",
                "protos/bstream.proto",
                "protos/sol_block.proto",
            ],
            &["protos/"],
        )
        .unwrap();
}
