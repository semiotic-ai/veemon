use anyhow::Context;
use bytes::Bytes;
use decoder::handle_buf;
use object_store::{
    aws::AmazonS3Builder, gcp::GoogleCloudStorageBuilder, local::LocalFileSystem, path::Path,
    ObjectStore,
};
use std::sync::Arc;
use thiserror::Error;
use url::Url;

use sf_protos::ethereum::r#type::v2::Block;

pub fn new<S: AsRef<str>>(store_url: S) -> Result<Store, anyhow::Error> {
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

    match url.scheme() {
        "s3" => {
            let bucket = url.host_str().ok_or_else(|| anyhow::anyhow!("No bucket"))?;
            let path = url.path();

            let store = AmazonS3Builder::new()
                .with_bucket_name(bucket.to_string())
                .build()?;

            Ok(Store {
                store: Arc::new(store),
                base: match path.starts_with("/") {
                    false => path.to_string(),
                    true => path[1..].to_string(),
                },
            })
        }
        "gs" => {
            let bucket = url.host_str().ok_or_else(|| anyhow::anyhow!("No bucket"))?;
            let path = url.path();

            let store = GoogleCloudStorageBuilder::new()
                .with_bucket_name(bucket.to_string())
                .build()?;

            Ok(Store {
                store: Arc::new(store),
                base: match path.starts_with("/") {
                    false => path.to_string(),
                    true => path[1..].to_string(),
                },
            })
        }
        "file" => {
            let store = LocalFileSystem::new_with_prefix(url.path()).context("new local store")?;

            Ok(Store {
                store: Arc::new(store),
                base: "".to_string(),
            })
        }
        _ => Err(anyhow::anyhow!("Unsupported scheme: {}", url.scheme()))?,
    }
}

#[derive(Clone)]
pub struct Store {
    store: Arc<dyn ObjectStore>,
    base: String,
}

impl Store {
    pub async fn read_blocks(
        &self,
        path: String,
        options: ReadOptions,
    ) -> Result<Vec<Block>, ReadError> {
        let content = self.store.get(&self.join_path(path)).await?;
        let bytes: Bytes = content.bytes().await.unwrap();

        handle_from_bytes(bytes, options.decompress()).await
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

async fn handle_from_bytes(bytes: Bytes, decompress: bool) -> Result<Vec<Block>, ReadError> {
    handle_buf(bytes.as_ref(), Some(decompress)).map_err(|e| ReadError::DecodeError(e.to_string()))
}

// async fn fake_handle_from_stream(
//     mut stream: BoxStream<'static, Result<Bytes, object_store::Error>>,
//     decompress: bool,
// ) -> Result<Vec<Block>, ReadError> {
//     use futures::stream::TryStreamExt; // for `try_next`

//     let mut file = tokio::fs::OpenOptions::new()
//         .write(true)
//         .create(true)
//         .truncate(true)
//         .open("/tmp/temp_block.dbin.zst")
//         .await
//         .expect("demo code, no file would be use when flat_file_decoders will be updated");

//     while let Some(item) = stream.try_next().await? {
//         file.write_all(&item)
//             .await
//             .expect("demo code, unable to write to temp file");
//     }

//     file.sync_all()
//         .await
//         .expect("demo code, unable to sync temp file");
//     drop(file);

//     Ok(decode_flat_files(
//         "/tmp/temp_block.dbin.zst".to_string(),
//         None,
//         None,
//         Some(decompress),
//     )
//     .expect("demo code, deal with error nicely"))
// }
