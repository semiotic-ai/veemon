//! # Rust Firehose Client
//!
//! Rust implementation of a client for the [StreamingFast Firehose](https://firehose.streamingfast.io/)
//! gRPC Fetch `Block` and Stream `Block`s APIs.
//!
//! ## Fetching an Ethereum Block
//!
//! ```no_run
//! # use firehose_client::{Chain, FirehoseClient};
//! # use firehose_protos::EthBlock as Block;
//! # #[tokio::main]
//! # async fn main() -> Result<(), firehose_protos::ProtosError> {
//! let mut client = FirehoseClient::new(Chain::Ethereum);
//!
//! if let Some(response) = client.fetch_block(20672593).await.unwrap().ok() {
//!     let block = Block::try_from(response.into_inner())?;
//!     assert_eq!(block.number, 20672593);
//!     assert_eq!(
//!         format!("0x{}", hex::encode(block.hash)).as_str(),
//!         "0xea48ba1c8e38ea586239e9c5ec62949ddd79404c6006c099bb02a8b22ddd18e4"
//!     );
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Streaming Ethereum Blocks
//!
//! ```no_run
//! # use firehose_client::{Chain, FirehoseClient};
//! # use futures::StreamExt;
//! # #[tokio::main]
//! # async fn main() -> Result<(), firehose_protos::ProtosError> {
//! const TOTAL_BLOCKS: u64 = 8192;
//! const START_BLOCK: u64 = 19581798;
//!
//! let mut client = FirehoseClient::new(Chain::Ethereum);
//! let mut stream = client
//!     .stream_blocks(START_BLOCK, TOTAL_BLOCKS)
//!     .await
//!     .unwrap();
//!
//! while let Some(block) = stream.next().await {
//!     // Do Something with the extracted stream of blocks.
//! }
//! # Ok(())
//! # }
//! ```
//!

mod client;
mod error;
mod tls;

pub use crate::client::{Chain, FirehoseClient};
