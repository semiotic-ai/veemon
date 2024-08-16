use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Error in decoding block: {0}")]
    DecodeError(#[from] prost::DecodeError),
    #[error("Null block field in response")]
    NullBlock,
    #[error("Error in fetching block: {0}")]
    TransportError(#[from] tonic::transport::Error),
}
