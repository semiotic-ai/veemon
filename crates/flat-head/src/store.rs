use anyhow::Context;
use bytes::Bytes;
use flat_files_decoder::{decompression::Decompression, handle_buf};
use object_store::{
    aws::AmazonS3Builder, gcp::GoogleCloudStorageBuilder, http::HttpBuilder,
    local::LocalFileSystem, path::Path, ClientOptions, ObjectStore,
};
use std::sync::Arc;
use thiserror::Error;
use url::Url;

use firehose_protos::ethereum_v2::Block;

pub fn new<S: AsRef<str>>(
    store_url: S,
    decompress: Decompression,
    compatible: Option<String>,
) -> Result<Store, anyhow::Error> {
    let store_url = store_url.as_ref();
    let url = match Url::parse(store_url) {
        Ok(url) => url,
        Err(url::ParseError::RelativeUrlWithoutBase) => {
            let absolute_path = std::fs::canonicalize(store_url)
                .map_err(|e| anyhow::anyhow!("Invalid store URL: {}: {}", store_url, e))?;

            Url::parse(&format!("file://{}", absolute_path.to_string_lossy()))
                .with_context(|| format!("Invalid store URL: {}", store_url))?
        }
        Err(e) => Err(e).with_context(|| format!("Invalid store URL: {}", store_url))?,
    };

    let path = url.path();

    let base_path = match path.starts_with('/') {
        false => path.to_string(),
        true => path[1..].to_string(),
    };

    match url.scheme() {
        "http" | "https" => {
            //TODO: setup a flag for s3 compatible http APIs such as seaweed fs.
            let scheme = url.scheme();

            let endpoint = match url.host_str() {
                Some(host) => {
                    let port = url.port_or_known_default();
                    format!("{}://{}:{}", scheme, host, port.unwrap())
                }
                None => return Err(anyhow::anyhow!("invalid url format")),
            };

            let store: Arc<dyn ObjectStore> = match compatible.as_deref() {
                Some("s3") => {
                    let s3_store = AmazonS3Builder::new()
                        .with_endpoint(endpoint.to_string())
                        .with_bucket_name(url.path()[1..].to_string())
                        .with_allow_http(scheme == "http")
                        .with_access_key_id("any")
                        .with_secret_access_key("any")
                        .build()
                        .context("Failed to build S3 store")?;

                    Arc::new(s3_store) as Arc<dyn ObjectStore>
                }
                _ => {
                    // Fallback to the HttpBuilder
                    let http_store = HttpBuilder::new()
                        .with_url(endpoint.to_string())
                        .with_client_options(ClientOptions::new().with_allow_http(scheme == "http"))
                        .build()
                        .context("Failed to build HTTP store")?;

                    Arc::new(http_store) as Arc<dyn ObjectStore>
                }
            };

            Ok(Store {
                store: Arc::new(store),
                base: "".to_string(),
                decompress,
            })
        }
        "s3" => {
            let bucket: &str = url.host_str().ok_or_else(|| anyhow::anyhow!("No bucket"))?;

            let store = AmazonS3Builder::new()
                .with_bucket_name(bucket.to_string())
                .build()?;

            Ok(Store {
                store: Arc::new(store),
                base: base_path,
                decompress,
            })
        }
        "gs" => {
            let bucket = url.host_str().ok_or_else(|| anyhow::anyhow!("No bucket"))?;

            let store = GoogleCloudStorageBuilder::new()
                .with_bucket_name(bucket.to_string())
                .build()?;

            Ok(Store {
                store: Arc::new(store),
                base: base_path,
                decompress,
            })
        }
        "file" => {
            let store = LocalFileSystem::new_with_prefix(url.path()).context("new local store")?;

            Ok(Store {
                store: Arc::new(store),
                base: "".to_string(),
                decompress,
            })
        }
        _ => Err(anyhow::anyhow!("Unsupported scheme: {}", url.scheme()))?,
    }
}

#[derive(Clone)]
pub struct Store {
    store: Arc<dyn ObjectStore>,
    base: String,
    decompress: Decompression,
}

impl Store {
    pub async fn read_blocks(&self, path: String) -> Result<Vec<Block>, ReadError> {
        let content = self.store.get(&self.join_path(path)).await?;
        let bytes: Bytes = content.bytes().await.unwrap();
        handle_from_bytes(bytes, self.decompress).await
    }

    fn join_path(&self, path: String) -> Path {
        Path::from(format!("{}/{}", self.base, path.trim_start_matches('/')))
    }
}

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Path '{0}' not found")]
    NotFound(String),
    #[error("Storage error: {0}")]
    Storage(#[from] object_store::Error),
    #[error("Decode error: {0}")]
    DecodeError(String), // Or directly use DecodeError if it implements `std::error::Error`
}

pub struct ReadOptions {
    pub decompress: Option<bool>,
}

impl ReadOptions {
    pub fn decompress(&self) -> bool {
        self.decompress.unwrap_or(true)
    }
}

async fn handle_from_bytes(
    bytes: Bytes,
    decompress: Decompression,
) -> Result<Vec<Block>, ReadError> {
    handle_buf(bytes.as_ref(), decompress).map_err(|e| ReadError::DecodeError(e.to_string()))
}
