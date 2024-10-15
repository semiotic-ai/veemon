use http::uri::InvalidUri;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("gRPC error: {0}")]
    GRpc(#[from] tonic::transport::Error),

    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] InvalidUri),

    #[error("Missing environment variable: {0}")]
    MissingEnvVar(#[from] dotenvy::Error),

    #[error("{0}")]
    TonicStatus(#[from] tonic::Status),
}
