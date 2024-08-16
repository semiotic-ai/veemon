mod error;
pub mod execution_layer_types {
    tonic::include_proto!("sf.ethereum.r#type.v2");
}
pub mod execution_layer_firehose {
    tonic::include_proto!("sf.firehose.v2");
}
pub mod execution_layer_client;
