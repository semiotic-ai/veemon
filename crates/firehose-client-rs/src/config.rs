use http::uri::InvalidUri;
use thiserror::Error;
use tonic::transport::Uri;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] InvalidUri),
    
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(#[from] dotenvy::Error),
}

pub fn firehose_ethereum_uri() -> Result<Uri, ConfigError> {
    dotenvy::dotenv()?;

    let url = dotenvy::var("FIREHOSE_ETHEREUM_URL")?;
    let port = dotenvy::var("FIREHOSE_ETHEREUM_PORT")?;

    Ok(format!("{}:{}", url, port).parse::<Uri>()?)
}
