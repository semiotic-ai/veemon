// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeaderAccumulator {
    #[prost(string, tag="1")]
    pub root: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Hashes {
    #[prost(string, repeated, tag="1")]
    pub hashes: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(string, tag="1")]
    pub block_hash: ::prost::alloc::string::String,
    #[prost(uint64, tag="2")]
    pub block_number: u64,
    #[prost(message, optional, tag="3")]
    pub total_difficulty: ::core::option::Option<::substreams_ethereum::pb::eth::v2::BigInt>,
}
// @@protoc_insertion_point(module)
