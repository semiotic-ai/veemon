pub mod channel {
    use tonic::transport::{Channel, Uri};

    pub async fn build_and_connect_channel(uri: Uri) -> Result<Channel, tonic::transport::Error> {
        if uri.scheme_str() != Some("https") {
            return Channel::builder(uri).connect().await;
        }

        let config = super::tls::config();

        Channel::builder(uri).tls_config(config)?.connect().await
    }
}

pub mod endpoint {
    use tonic::transport::Uri;

    use super::error::ConfigError;

    pub enum Firehose {
        Ethereum,
        Beacon,
    }

    impl Firehose {
        pub fn uri_from_env(&self) -> Result<Uri, ConfigError> {
            dotenvy::dotenv()?;

            let (url, port) = match self {
                Self::Ethereum => (
                    dotenvy::var("FIREHOSE_ETHEREUM_URL")?,
                    dotenvy::var("FIREHOSE_ETHEREUM_PORT")?,
                ),
                Self::Beacon => (
                    dotenvy::var("FIREHOSE_BEACON_URL")?,
                    dotenvy::var("FIREHOSE_BEACON_PORT")?,
                ),
            };

            Ok(format!("{}:{}", url, port).parse::<Uri>()?)
        }
    }
}

pub mod error {
    use http::uri::InvalidUri;
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum ConfigError {
        #[error("Invalid URI: {0}")]
        InvalidUri(#[from] InvalidUri),

        #[error("Missing environment variable: {0}")]
        MissingEnvVar(#[from] dotenvy::Error),
    }
}

pub mod tls {
    use tonic::transport::ClientTlsConfig;

    pub fn config() -> ClientTlsConfig {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install rustls crypto provider");

        ClientTlsConfig::new()
            .with_native_roots()
            .assume_http2(true)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use sf_protos::{
        beacon::r#type::v1::block::Body,
        ethereum::r#type::v2::Block,
        firehose::v2::{fetch_client::FetchClient, stream_client::StreamClient},
    };
    use tonic::metadata::MetadataValue;

    use crate::{
        client::{channel::build_and_connect_channel, endpoint::Firehose},
        request::{create_blocks_request, create_request, BlocksRequested},
    };

    /// Demonstrates how to fetch a single block from Beacon Firehose, using the `Fetch` API.
    #[tokio::test]
    async fn test_firehose_beacon_fetch_block_by_slot() {
        // Show matching data from execution layer and beacon chain
        let uri = Firehose::Ethereum.uri_from_env().unwrap();

        let channel = build_and_connect_channel(uri).await.unwrap();

        let mut client = FetchClient::new(channel);

        // This is the block number for the execution block we want to fetch
        let mut request = create_request(20672593);

        if let Ok(api_key) = std::env::var("ETHEREUM_API_KEY") {
            let api_key_header = MetadataValue::from_str(&api_key).expect("Invalid API key format");
            request.metadata_mut().insert("x-api-key", api_key_header);
        }

        let response = client.block(request).await.unwrap();

        let block = Block::try_from(response.into_inner()).unwrap();

        assert_eq!(block.number, 20672593);
        assert_eq!(
            format!("0x{}", hex::encode(block.hash)).as_str(),
            "0xea48ba1c8e38ea586239e9c5ec62949ddd79404c6006c099bb02a8b22ddd18e4"
        );

        let uri = Firehose::Beacon.uri_from_env().unwrap();

        let channel = build_and_connect_channel(uri).await.unwrap();

        let mut client = FetchClient::new(channel);

        // This is the slot number for the Beacon block we want to fetch, but right now
        // we don't have a way to map the block number of the execution block to the slot number
        // of the Beacon block.
        let mut request = create_request(9881091);

        if let Ok(api_key) = std::env::var("BEACON_API_KEY") {
            let api_key_header = MetadataValue::from_str(&api_key).expect("Invalid API key format");
            request.metadata_mut().insert("x-api-key", api_key_header);
        }

        let response = client.block(request).await.unwrap();

        let block = sf_protos::beacon::r#type::v1::Block::try_from(response.into_inner()).unwrap();

        assert_eq!(block.slot, 9881091);

        let body = block.body.as_ref().unwrap();

        match body {
            Body::Deneb(body) => {
                let execution_payload = body.execution_payload.as_ref().unwrap();

                let block_hash = &execution_payload.block_hash;

                assert_eq!(
                    format!("0x{}", hex::encode(block_hash)).as_str(),
                    "0xea48ba1c8e38ea586239e9c5ec62949ddd79404c6006c099bb02a8b22ddd18e4"
                );

                let block_number = execution_payload.block_number;

                assert_eq!(block_number, 20672593);
            }
            _ => unimplemented!(),
        };
    }

    /// Demonstrates how to fetch a single block from Ethereum firehose, using the `FetchClient`.
    #[tokio::test]
    async fn test_firehose_ethereum_fetch_block() {
        let uri = Firehose::Ethereum.uri_from_env().unwrap();

        let channel = build_and_connect_channel(uri).await.unwrap();

        let mut client = FetchClient::new(channel);

        let mut request = create_request(20672593);

        if let Ok(api_key) = std::env::var("ETHEREUM_API_KEY") {
            let api_key_header = MetadataValue::from_str(&api_key).expect("Invalid API key format");
            request.metadata_mut().insert("x-api-key", api_key_header);
        }

        let response = client.block(request).await.unwrap();

        let block = Block::try_from(response.into_inner()).unwrap();

        assert_eq!(block.number, 20672593);
        assert_eq!(
            format!("0x{}", hex::encode(block.hash)).as_str(),
            "0xea48ba1c8e38ea586239e9c5ec62949ddd79404c6006c099bb02a8b22ddd18e4"
        );
    }

    /// Temporary test to demonstrate how to stream a range of blocks from Firehose Ethereum
    #[tokio::test]
    async fn test_firehose_ethereum_stream_blocks() {
        // Testing this so far without proper benchmarking, the time taken to fetch the blocks
        // grows linearly with the number of TOTAL_BLOCKS requested, to around 20 minutes for 8192 blocks!
        const TOTAL_BLOCKS: u64 = 10;
        const START_BLOCK: u64 = 19581798;

        let uri = Firehose::Ethereum.uri_from_env().unwrap();

        let channel = build_and_connect_channel(uri).await.unwrap();

        let mut client = StreamClient::new(channel);

        let end_block = START_BLOCK + TOTAL_BLOCKS - 1;

        let mut request =
            create_blocks_request(START_BLOCK as i64, end_block, BlocksRequested::FinalOnly);

        if let Ok(api_key) = std::env::var("ETHEREUM_API_KEY") {
            let api_key_header = MetadataValue::from_str(&api_key).expect("Invalid API key format");
            request.metadata_mut().insert("x-api-key", api_key_header);
        }

        let response = client.blocks(request).await.unwrap();
        let mut stream_inner = response.into_inner();

        let mut blocks: Vec<Block> = Vec::new();

        while let Ok(Some(block_msg)) = stream_inner.message().await {
            let block = Block::try_from(block_msg).unwrap();
            blocks.push(block);
        }

        // For now, just using this to signal that the test has completed
        eprintln!("Number of elements: {}", blocks.len());
    }
}
