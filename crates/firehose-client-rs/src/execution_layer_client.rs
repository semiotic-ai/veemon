use tonic::transport::{Channel, Uri};

pub async fn build_and_connect_channel(endpoint: Uri) -> Result<Channel, tonic::transport::Error> {
    Channel::builder(endpoint).connect().await
}

#[cfg(test)]
mod tests {
    use sf_protos::{
        ethereum::r#type::v2::Block,
        firehose::v2::{fetch_client::FetchClient, stream_client::StreamClient},
    };

    use crate::{
        config::firehose_ethereum_uri,
        request::{create_block_request, create_blocks_request, BlocksRequested},
    };

    use super::*;

    /// Demonstrates how to fetch a single block from Ethereum firehose, using the `FetchClient`.
    #[tokio::test]
    async fn test_grpc_fetch_block() {
        let uri = firehose_ethereum_uri().unwrap();

        let channel = build_and_connect_channel(uri).await.unwrap();

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

        let uri = firehose_ethereum_uri().unwrap();

        let channel = build_and_connect_channel(uri).await.unwrap();

        let mut client = StreamClient::new(channel);

        let end_block = START_BLOCK + TOTAL_BLOCKS - 1;

        let request =
            create_blocks_request(START_BLOCK as i64, end_block, BlocksRequested::FinalOnly);

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
