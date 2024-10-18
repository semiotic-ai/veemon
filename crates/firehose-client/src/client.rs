use std::str::FromStr;

use crate::error::ClientError;
use dotenvy::{dotenv, var};
use sf_protos::{
    beacon::r#type::v1::Block as FirehoseBeaconBlock,
    ethereum::r#type::v2::Block as FirehoseEthBlock,
    firehose::v2::{
        fetch_client::FetchClient,
        single_block_request::{BlockNumber, Reference},
        stream_client::StreamClient,
        Request, SingleBlockRequest, SingleBlockResponse,
    },
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{
    transport::{Channel, Uri},
    Response, Status,
};
use tracing::{error, info, trace};

pub struct FirehoseClient {
    chain: Chain,
    fetch_client: Option<FetchClient<Channel>>,
    stream_client: Option<StreamClient<Channel>>,
}

impl FirehoseClient {
    pub fn new(chain: Chain) -> Self {
        Self {
            chain,
            fetch_client: None,
            stream_client: None,
        }
    }

    /// The inner [`Result`] is the Firehose response, which can be either a [`Response`] or a [`Status`],
    /// which is needed for handling empty slots on the Beacon chain.
    pub async fn fetch_block(
        &mut self,
        number: u64,
    ) -> Result<Result<Response<SingleBlockResponse>, Status>, ClientError> {
        if self.fetch_client.is_none() {
            self.fetch_client = Some(fetch_client(self.chain).await?);
        }
        let mut request = create_single_block_fetch_request(number);

        request.insert_api_key_if_provided(self.chain);

        info!("Requesting block number:\n\t{}", number);
        Ok(self.fetch_client.as_mut().unwrap().block(request).await)
    }

    /// The tonic docs encourage cloning the client.
    pub async fn get_streaming_client(&mut self) -> Result<StreamClient<Channel>, ClientError> {
        let client = if let Some(client) = self.stream_client.clone() {
            client
        } else {
            self.stream_client = Some(stream_client(self.chain).await?);
            self.stream_client.clone().unwrap()
        };
        Ok(client)
    }

    /// Stream a block range of Beacon blocks, with a retry mechanism if the stream cuts off
    /// before the total number of blocks requested is reached, and accounting for missed slots.
    pub async fn stream_beacon_with_retry(
        &mut self,
        start: u64,
        total: u64,
    ) -> Result<impl futures::Stream<Item = FirehoseBeaconBlock>, ClientError> {
        let (tx, rx) = tokio::sync::mpsc::channel::<FirehoseBeaconBlock>(8192);

        let chain = self.chain;
        let client = self.get_streaming_client().await?;

        tokio::spawn(async move {
            let mut blocks = 0;
            let mut last_valid_slot: Option<u64> = None;
            let mut last_valid_block: Option<FirehoseBeaconBlock> = None;

            while blocks < total {
                let mut client = client.clone();
                let request = create_blocks_streaming_request(
                    chain,
                    start + blocks,
                    start + total - 1,
                    BlocksRequested::All,
                );
                match client.blocks(request).await {
                    Ok(response) => {
                        let mut stream_inner = response.into_inner();
                        while let Ok(Some(block_msg)) = stream_inner.message().await {
                            if blocks % 100 == 0 {
                                trace!("Blocks fetched: {}", blocks);
                            }
                            match FirehoseBeaconBlock::try_from(block_msg) {
                                Ok(block) => {
                                    if let Some(last_slot) = last_valid_slot {
                                        let missed_slots = block.slot.saturating_sub(last_slot + 1);
                                        if missed_slots > 0 {
                                            trace!("Missed block at slot: {}", start + blocks);

                                            let last_block = last_valid_block.take().unwrap();
                                            let tx = tx.clone();
                                            for _ in 0..missed_slots {
                                                blocks += 1;
                                                tx.send(last_block.clone()).await.unwrap();
                                            }
                                        }
                                    }
                                    last_valid_slot = Some(block.slot);
                                    last_valid_block = Some(block.clone());

                                    blocks += 1;
                                    tx.clone().send(block).await.unwrap();
                                }
                                Err(e) => {
                                    error!("Failed to convert block message to block: {e}");
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to get blocks stream: {:?}", e.code());
                        break;
                    }
                };
            }
        });

        Ok(ReceiverStream::new(rx))
    }

    pub async fn stream_ethereum_with_retry(
        &mut self,
        start: u64,
        total: u64,
    ) -> Result<impl futures::Stream<Item = FirehoseEthBlock>, ClientError> {
        let (tx, rx) = tokio::sync::mpsc::channel::<FirehoseEthBlock>(8192);

        let chain = self.chain;
        let client = self.get_streaming_client().await?;

        tokio::spawn(async move {
            let mut blocks = 0;

            while blocks < total {
                let mut client = client.clone();
                let request = create_blocks_streaming_request(
                    chain,
                    start + blocks,
                    start + total - 1,
                    BlocksRequested::All,
                );
                let response = client.blocks(request).await.unwrap();
                let mut stream_inner = response.into_inner();
                while let Ok(Some(block_msg)) = stream_inner.message().await {
                    if blocks % 100 == 0 && blocks != 0 {
                        trace!("Blocks fetched: {}", blocks);
                    }
                    match FirehoseEthBlock::try_from(block_msg) {
                        Ok(block) => {
                            blocks += 1;
                            tx.clone().send(block).await.unwrap();
                        }
                        Err(e) => {
                            panic!("Failed to convert block message to block: {e}");
                        }
                    }
                }
            }
        });

        Ok(ReceiverStream::new(rx))
    }
}

async fn build_and_connect_channel(uri: Uri) -> Result<Channel, tonic::transport::Error> {
    if uri.scheme_str() != Some("https") {
        return Channel::builder(uri).connect().await;
    }

    let config = crate::tls::config();

    Channel::builder(uri)
        .tls_config(config.clone())?
        .connect()
        .await
}

fn create_blocks_streaming_request(
    chain: Chain,
    start_block_num: u64,
    stop_block_num: u64,
    blocks_requested: BlocksRequested,
) -> tonic::Request<Request> {
    let mut request = tonic::Request::new(Request {
        start_block_num: start_block_num as i64,
        stop_block_num,
        final_blocks_only: blocks_requested.into(),
        ..Default::default()
    });
    request.insert_api_key_if_provided(chain);
    request
}

async fn fetch_client(firehose: Chain) -> Result<FetchClient<Channel>, ClientError> {
    Ok(FetchClient::new({
        let uri = firehose.uri_from_env()?;
        build_and_connect_channel(uri).await?
    }))
}

async fn stream_client(firehose: Chain) -> Result<StreamClient<Channel>, ClientError> {
    Ok(StreamClient::new({
        let uri = firehose.uri_from_env()?;
        build_and_connect_channel(uri).await?
    }))
}

pub enum BlocksRequested {
    All,
    FinalOnly,
}

impl From<BlocksRequested> for bool {
    fn from(blocks_requested: BlocksRequested) -> bool {
        match blocks_requested {
            BlocksRequested::All => false,
            BlocksRequested::FinalOnly => true,
        }
    }
}

/// Create a [`SingleBlockRequest`] for the given *number*.
/// Number is slot number for beacon blocks.
fn create_single_block_fetch_request(num: u64) -> tonic::Request<SingleBlockRequest> {
    tonic::Request::new(SingleBlockRequest {
        reference: Some(Reference::BlockNumber(BlockNumber { num })),
        ..Default::default()
    })
}

trait FirehoseRequest {
    fn insert_api_key_if_provided(&mut self, endpoint: Chain);
}

impl<T> FirehoseRequest for tonic::Request<T> {
    fn insert_api_key_if_provided(&mut self, endpoint: Chain) {
        insert_api_key_if_provided(self, endpoint);
    }
}

fn insert_api_key_if_provided<T>(request: &mut tonic::Request<T>, chain: Chain) {
    if let Ok(api_key) = var(chain.api_key_env_var_as_str()) {
        let api_key_header =
            tonic::metadata::MetadataValue::from_str(&api_key).expect("Invalid API key format");
        request.metadata_mut().insert("x-api-key", api_key_header);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Chain {
    Ethereum,
    Beacon,
}

impl Chain {
    fn api_key_env_var_as_str(&self) -> &str {
        match self {
            Self::Beacon => "BEACON_API_KEY",
            Self::Ethereum => "ETHEREUM_API_KEY",
        }
    }

    fn uri_from_env(&self) -> Result<Uri, ClientError> {
        dotenv()?;

        let (url, port) = match self {
            Self::Ethereum => (
                var("FIREHOSE_ETHEREUM_URL")?,
                var("FIREHOSE_ETHEREUM_PORT")?,
            ),
            Self::Beacon => (var("FIREHOSE_BEACON_URL")?, var("FIREHOSE_BEACON_PORT")?),
        };

        Ok(format!("{}:{}", url, port).parse::<Uri>()?)
    }
}
