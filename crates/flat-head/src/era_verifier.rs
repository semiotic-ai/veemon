use flat_files_decoder::compression::Compression;
use futures::stream::{FuturesOrdered, StreamExt};
use tokio::task;

use firehose_protos::ethereum_v2::Block;
use header_accumulator::{EraValidator, ExtHeaderRecord};
use tokio::sync::mpsc;
use tree_hash::Hash256;
use trin_validation::accumulator::PreMergeAccumulator;

use crate::store::{self, Store};
pub const MAX_EPOCH_SIZE: usize = 8192;
pub const FINAL_EPOCH: usize = 1896;
pub const MERGE_BLOCK: usize = 15537394;

/// verifies flat flies stored in directory against a header accumulator
///
pub async fn verify_eras(
    store_url: String,
    macc: PreMergeAccumulator,
    compatible: Option<String>,
    start_epoch: usize,
    end_epoch: Option<usize>,
    decompress: Compression,
) -> Result<Vec<Hash256>, anyhow::Error> {
    let mut validated_epochs = Vec::new();
    let (tx, mut rx) = mpsc::channel(5);

    let blocks_store: store::Store =
        store::new(store_url, decompress, compatible).expect("failed to create blocks store");

    for epoch in start_epoch..=end_epoch.unwrap_or(start_epoch + 1) {
        let tx = tx.clone();
        let era_validator: EraValidator = macc.clone().into();
        let store = blocks_store.clone();

        task::spawn(async move {
            match get_blocks_from_store(epoch, &store, decompress).await {
                Ok(blocks) => {
                    let (successful_headers, _): (Vec<_>, Vec<_>) = blocks
                        .iter()
                        .map(ExtHeaderRecord::try_from)
                        .fold((Vec::new(), Vec::new()), |(mut succ, mut errs), res| {
                            match res {
                                Ok(header) => succ.push(header),
                                Err(e) => {
                                    // Log the error or handle it as needed
                                    eprintln!("Error converting block: {:?}", e);
                                    errs.push(e);
                                }
                            };
                            (succ, errs)
                        });

                    let epoch = successful_headers.try_into().unwrap();
                    let valid_epochs = era_validator.validate_era(&epoch).unwrap();

                    let _ = tx.send(valid_epochs).await;
                }
                Err(e) => eprintln!("Error fetching blocks for epoch {}: {:?}", epoch, e),
            }
        });
    }

    // Drop the original sender to close the channel once all senders are dropped
    drop(tx);

    // Process blocks as they arrive
    while let Some(epochs) = rx.recv().await {
        validated_epochs.push(epochs);
    }

    Ok(validated_epochs)
}

async fn get_blocks_from_store(
    epoch: usize,
    store: &Store,
    decompress: Compression,
) -> Result<Vec<Block>, anyhow::Error> {
    let start_100_block = epoch * MAX_EPOCH_SIZE;
    let end_100_block = (epoch + 1) * MAX_EPOCH_SIZE;

    let mut blocks = extract_100s_blocks(store, start_100_block, end_100_block, decompress).await?;

    if epoch < FINAL_EPOCH {
        blocks = blocks[0..MAX_EPOCH_SIZE].to_vec();
    } else {
        blocks = blocks[0..MERGE_BLOCK].to_vec();
    }

    Ok(blocks)
}

async fn extract_100s_blocks(
    store: &Store,
    start_block: usize,
    end_block: usize,
    decompress: Compression,
) -> Result<Vec<Block>, anyhow::Error> {
    // Flat files are stored in 100 block files
    // So we need to find the 100 block file that contains the start block and the 100 block file that contains the end block
    let start_100_block = (start_block / 100) * 100;
    let end_100_block = (((end_block as f32) / 100.0).ceil() as usize) * 100;

    let zst_extension = match decompress {
        Compression::Zstd => ".zst",
        Compression::None => "",
    };

    let mut futs = FuturesOrdered::new();

    for block_number in (start_100_block..end_100_block).step_by(100) {
        let block_file_name = format!("{:010}.dbin{}", block_number, zst_extension);
        futs.push_back(store.read_blocks(block_file_name))
    }

    let mut blocks_join = Vec::new();

    while let Some(res) = futs.next().await {
        match res {
            Ok(blocks) => blocks_join.extend(blocks),
            Err(e) => println!("{:?}", e),
        }
    }

    // Return only the requested blocks

    Ok(blocks_join[start_block - start_100_block..end_block - start_100_block].to_vec())
}
