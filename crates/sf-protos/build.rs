use prost_build::Config;
use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Configure prost to derive serde traits on specific types
    let mut config = Config::new();
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    // Map Google protobuf types to prost_wkt_types
    config.extern_path(".google.protobuf.Any", "::prost_wkt_types::Any");
    config.extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp");

    tonic_build::configure()
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("descriptors.bin"))
        .compile_with_config(
            config,
            &[
                "src/protos/block.proto",
                "src/protos/bstream.proto",
                "src/protos/firehose.proto",
                "src/protos/type.proto",
            ],
            &["src/protos/"],
        )
        .unwrap();
}
