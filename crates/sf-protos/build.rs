use prost_wkt_build::*;
use std::{env, io::Result, path::PathBuf};

fn configure_prost() -> prost_build::Config {
    let mut config = prost_build::Config::new();
    config
        .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
        .extern_path(".google.protobuf.Any", "::prost_wkt_types::Any")
        .extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp")
        .extern_path(".google.protobuf.Value", "::prost_wkt_types::Value");
    config
}

fn main() -> Result<()> {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let descriptor_file = out.join("descriptors.bin");

    let mut prost_build = configure_prost();
    prost_build
        .file_descriptor_set_path(&descriptor_file)
        .compile_protos(
            &["src/protos/block.proto", "src/protos/bstream.proto"],
            &["src/"],
        )?;

    let descriptor_bytes = std::fs::read(descriptor_file)?;
    let descriptor = FileDescriptorSet::decode(&descriptor_bytes[..])?;

    prost_wkt_build::add_serde(out, descriptor);

    Ok(())
}
