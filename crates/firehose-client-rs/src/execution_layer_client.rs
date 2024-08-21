use prost::Message;
use tonic::transport::{Channel, Uri};

use crate::{
    error::ClientError,
    execution_layer_firehose::{
        single_block_request::{BlockNumber, Reference},
        Request, Response, SingleBlockRequest, SingleBlockResponse,
    },
    execution_layer_types::Block,
};

pub async fn build_and_connect_channel(endpoint: Uri) -> Result<Channel, tonic::transport::Error> {
    Channel::builder(endpoint).connect().await
}

pub fn create_block_request(num: u64) -> SingleBlockRequest {
    SingleBlockRequest {
        reference: Some(Reference::BlockNumber(BlockNumber { num })),
        ..Default::default()
    }
}

pub fn create_blocks_request(start_block_num: i64, stop_block_num: u64) -> Request {
    Request {
        start_block_num,
        stop_block_num,
        final_blocks_only: true,
        ..Default::default()
    }
}

impl TryFrom<SingleBlockResponse> for Block {
    type Error = ClientError;

    fn try_from(response: SingleBlockResponse) -> Result<Self, Self::Error> {
        let any = response.block.ok_or(ClientError::NullBlock)?;
        let block = Block::decode(any.value.as_ref())?;
        Ok(block)
    }
}

impl TryFrom<Response> for Block {
    type Error = ClientError;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        let any = response.block.ok_or(ClientError::NullBlock)?;
        let block = Block::decode(any.value.as_ref())?;
        Ok(block)
    }
}

#[cfg(test)]
mod tests {
    use tonic::transport::Uri;

    use crate::execution_layer_firehose::{fetch_client::FetchClient, stream_client::StreamClient};

    use super::*;

    fn firehose_ethereum_uri() -> Uri {
        dotenvy::dotenv().unwrap();

        let url = dotenvy::var("FIREHOSE_ETHEREUM_URL").unwrap();
        let port = dotenvy::var("FIREHOSE_ETHEREUM_PORT").unwrap();

        format!("{}:{}", url, port).parse::<Uri>().unwrap()
    }

    /// Temporary test to demonstrate how to fetch a single block from the Ethereum firehose
    #[tokio::test]
    async fn test_grpc_fetch_block() {
        let uri = firehose_ethereum_uri();

        let channel = build_and_connect_channel(uri)
            .await
            .expect("Failed to connect to endpoint");

        // Use FetchClient to retrieve a single block
        let mut client = FetchClient::new(channel);

        let request = create_block_request(20562650);

        let response = client.block(request).await.unwrap();

        let block = Block::try_from(response.into_inner()).unwrap();

        assert_eq!(block.number, 20562650);
        assert_eq!(
            format!("0x{}", hex::encode(block.hash)).as_str(),
            "0xf218f8b4f7879b1c4a44b658a32d4a338db85c85c2916229d8b1c7728b448382"
        );
    }

    /// Temporary test to demonstrate how to stream a range of blocks from the Ethereum firehose
    #[tokio::test(flavor = "multi_thread")]
    async fn test_grpc_stream_blocks() {
        const TOTAL_BLOCKS: u64 = 8192;
        const START_BLOCK: u64 = 19581798;

        let uri = firehose_ethereum_uri();

        let channel = build_and_connect_channel(uri)
            .await
            .expect("Failed to connect to endpoint");

        let mut client = StreamClient::new(channel);

        let end_block = START_BLOCK + TOTAL_BLOCKS - 1;

        let request = create_blocks_request(START_BLOCK as i64, end_block);

        let response = client.blocks(request).await.unwrap();
        let mut stream_inner = response.into_inner();

        let mut blocks: Vec<Block> = Vec::new();

        while let Ok(Some(block_msg)) = stream_inner.message().await {
            let block = Block::try_from(block_msg).unwrap();
            blocks.push(block);
        }

        eprintln!("Number of elements: {}", blocks.len());

        let vec_size = std::mem::size_of_val(&blocks);
        let element_size = std::mem::size_of::<Block>();
        let total_elements_size = blocks.len() * element_size;
        let total_size = vec_size + total_elements_size;

        println!("Size of Vec<Block> structure: {} bytes", vec_size);
        println!("Size of each element: {} bytes", element_size);
        println!("Total size of elements: {} bytes", total_elements_size);
        println!("Total memory size of Vec<Block>: {} bytes", total_size);
    }
}
