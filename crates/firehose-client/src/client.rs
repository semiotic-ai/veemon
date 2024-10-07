pub mod channel {
    use sf_protos::firehose::v2::{fetch_client::FetchClient, stream_client::StreamClient};
    use tonic::transport::{Channel, Uri};

    use super::{endpoint::Firehose, error::ClientError};

    pub async fn build_and_connect_channel(uri: Uri) -> Result<Channel, tonic::transport::Error> {
        if uri.scheme_str() != Some("https") {
            return Channel::builder(uri).connect().await;
        }

        let config = super::tls::config();

        Channel::builder(uri).tls_config(config)?.connect().await
    }

    pub async fn fetch_client(firehose: Firehose) -> Result<FetchClient<Channel>, ClientError> {
        Ok(FetchClient::new({
            let execution_firehose_uri = firehose.uri_from_env()?;
            build_and_connect_channel(execution_firehose_uri).await?
        }))
    }

    pub async fn stream_client(firehose: Firehose) -> Result<StreamClient<Channel>, ClientError> {
        Ok(StreamClient::new({
            let execution_firehose_uri = firehose.uri_from_env()?;
            build_and_connect_channel(execution_firehose_uri).await?
        }))
    }
}

pub mod endpoint {
    use tonic::transport::Uri;

    use super::error::ClientError;

    pub enum Firehose {
        Ethereum,
        Beacon,
    }

    impl Firehose {
        pub fn uri_from_env(&self) -> Result<Uri, ClientError> {
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
    pub enum ClientError {
        #[error("gRPC error: {0}")]
        GRpc(#[from] tonic::transport::Error),

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
    use crate::{
        client::{
            channel::{build_and_connect_channel, fetch_client},
            endpoint::Firehose,
        },
        request::{create_blocks_request, create_request, BlocksRequested, FirehoseRequest},
    };
    use sf_protos::{
        beacon::{self, r#type::v1::block::Body},
        ethereum,
        firehose::v2::{fetch_client::FetchClient, stream_client::StreamClient},
    };

    /// Demonstrates how to fetch a single block from Beacon Firehose, using the `Fetch` API.
    #[tokio::test]
    async fn test_firehose_beacon_fetch_block_by_slot() {
        // Show matching data from execution layer and beacon chain
        let uri = Firehose::Ethereum.uri_from_env().unwrap();

        let channel = build_and_connect_channel(uri).await.unwrap();

        let mut execution_layer_client = FetchClient::new(channel);

        // This is the block number for the execution block we want to fetch
        let mut request = create_request(20672593);

        request.insert_api_key_if_provided(Firehose::Ethereum);

        let response = execution_layer_client.block(request).await.unwrap();

        let block = ethereum::r#type::v2::Block::try_from(response.into_inner()).unwrap();

        assert_eq!(block.number, 20672593);
        assert_eq!(
            format!("0x{}", hex::encode(block.hash)).as_str(),
            "0xea48ba1c8e38ea586239e9c5ec62949ddd79404c6006c099bb02a8b22ddd18e4"
        );

        let mut beacon_client = fetch_client(Firehose::Beacon).await.unwrap();

        // This is the slot number for the Beacon block we want to fetch, but right now
        // we don't have a way to map the block number of the execution block to the slot number
        // of the Beacon block.
        let mut request = create_request(9881091);

        request.insert_api_key_if_provided(Firehose::Beacon);

        let response = beacon_client.block(request).await.unwrap();

        let block = beacon::r#type::v1::Block::try_from(response.into_inner()).unwrap();

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

    /// Temporary test to demonstrate how to stream a range of blocks from Firehose Beacon
    #[tokio::test]
    async fn test_firehose_beacon_stream_blocks() {
        // Testing this so far without proper benchmarking, the time taken to fetch the blocks
        // grows linearly with the number of TOTAL_BLOCKS requested, to around 20 minutes for 8192 blocks!
        const TOTAL_SLOTS: u64 = 10;
        const START_SLOT: u64 = 9968872;

        let uri = Firehose::Beacon.uri_from_env().unwrap();

        let channel = build_and_connect_channel(uri).await.unwrap();

        let mut client = StreamClient::new(channel);

        let end_block = START_SLOT + TOTAL_SLOTS - 1;

        let mut request = create_blocks_request(START_SLOT, end_block, BlocksRequested::FinalOnly);

        request.insert_api_key_if_provided(Firehose::Beacon);

        let response = client.blocks(request).await.unwrap();
        let mut stream_inner = response.into_inner();

        let mut blocks: Vec<beacon::r#type::v1::Block> = Vec::with_capacity(TOTAL_SLOTS as usize);

        while let Ok(Some(block_msg)) = stream_inner.message().await {
            let block = beacon::r#type::v1::Block::try_from(block_msg).unwrap();
            blocks.push(block);
        }

        let slot_hash_map = blocks
            .iter()
            .map(|block| (block.slot, hex::encode(block.root.to_owned())))
            // Use BTreeMap for deterministic order.
            .collect::<std::collections::BTreeMap<u64, String>>();

        // For now, just using this to signal that the test has completed
        assert_eq!(blocks.len(), TOTAL_SLOTS as usize);
        insta::assert_debug_snapshot!(slot_hash_map, @r###"
        {
            9968872: "93888f0ef50b9b35bfa594d0971dfbffe5692385fc17730af6b2321b6695095f",
            9968873: "a149d7e490cfc7109700e47e77c521f94ecb585320aadeb4339eca361b124154",
            9968874: "f6e824c8f4e79da5f0ee59d2642073e23dda4c9ebe62f84f26a18f299ec1cfbc",
            9968875: "520db3414ef67d9280cf99c15195e3758d0db9b651d98b775818e530773de002",
            9968876: "02090ac39348b84f3022abe162a0c715898436650499750f240ec7eae8afd5f5",
            9968877: "937790abb3f73f1c9f6f4a6c97879bce0ebd0fb678d0af65f08338fd447bba6f",
            9968878: "61c3f4ac768c4c1f9add321aef3144f7a1417650c43251a0c65b58fd307e6248",
            9968879: "740a5fd2bff975410b70a4e77aea5ba71610813bb76f6d5e54eb9ee6748642e6",
            9968880: "f07f09b96fd0c212a95782315ec61f747d5d32173cd311920fd6af4b1e05aa9d",
            9968881: "4da93e355271c2edde13b4b72641f6111e50a635497250a3fbf650d38eee5f0e",
        }
        "###);
    }

    /// Demonstrates how to fetch a single block from Ethereum firehose, using the `FetchClient`.
    #[tokio::test]
    async fn test_firehose_ethereum_fetch_block() {
        let uri = Firehose::Ethereum.uri_from_env().unwrap();

        let channel = build_and_connect_channel(uri).await.unwrap();

        let mut client = FetchClient::new(channel);

        let mut request = create_request(20672593);

        request.insert_api_key_if_provided(Firehose::Ethereum);

        let response = client.block(request).await.unwrap();

        let block = ethereum::r#type::v2::Block::try_from(response.into_inner()).unwrap();

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

        let mut request = create_blocks_request(START_BLOCK, end_block, BlocksRequested::FinalOnly);

        request.insert_api_key_if_provided(Firehose::Ethereum);

        let response = client.blocks(request).await.unwrap();
        let mut stream_inner = response.into_inner();

        let mut blocks: Vec<ethereum::r#type::v2::Block> = Vec::new();

        while let Ok(Some(block_msg)) = stream_inner.message().await {
            let block = ethereum::r#type::v2::Block::try_from(block_msg).unwrap();
            blocks.push(block);
        }

        // For now, just using this to signal that the test has completed
        assert_eq!(blocks.len(), TOTAL_BLOCKS as usize);
    }
}
