use http::uri::InvalidUri;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Missing environment variable: {0}")]
    EnvVarMissing(#[from] dotenvy::Error),

    #[error("gRPC error: {0}")]
    GRpc(#[from] tonic::transport::Error),

    #[error("{0}")]
    TonicStatus(#[from] tonic::Status),

    #[error("Invalid URI: {0}")]
    UriInvalid(#[from] InvalidUri),
}
