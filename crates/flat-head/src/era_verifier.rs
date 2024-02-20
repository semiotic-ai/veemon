use decoder::{decode_flat_files, sf::ethereum::r#type::v2::Block};
use header_accumulator::{era_validator::era_validate, errors::EraValidateError};

pub const MAX_EPOCH_SIZE: usize = 8192;
pub const FINAL_EPOCH: usize = 01896;
pub const MERGE_BLOCK: usize = 15537394;

/// verifies flat flies stored in directory against a header accumulator
///
pub fn verify_eras(
    directory: &String,
    master_acc_file: Option<&String>,
    start_epoch: usize,
    end_epoch: Option<usize>,
) -> Result<Vec<usize>, EraValidateError> {
    let mut validated_epochs = Vec::new();

    for epoch in start_epoch..end_epoch.unwrap_or(start_epoch + 1) {
        let blocks = get_blocks_from_dir(epoch, directory)?;
        let root = era_validate(blocks, master_acc_file, epoch, Some(epoch + 1))?;
        validated_epochs.extend(root);
    }

    return Ok(validated_epochs);
}

fn get_blocks_from_dir(epoch: usize, directory: &String) -> Result<Vec<Block>, EraValidateError> {
    let start_100_block = epoch * MAX_EPOCH_SIZE;
    let end_100_block = (epoch + 1) * MAX_EPOCH_SIZE;

    let mut blocks = extract_100s_blocks(directory, start_100_block, end_100_block)?;

    if epoch < FINAL_EPOCH {
        blocks = blocks[0..MAX_EPOCH_SIZE].to_vec();
    } else {
        blocks = blocks[0..MERGE_BLOCK].to_vec();
    }

    Ok(blocks)
}

fn extract_100s_blocks(
    directory: &String,
    start_block: usize,
    end_block: usize,
) -> Result<Vec<Block>, EraValidateError> {
    // Flat files are stored in 100 block files
    // So we need to find the 100 block file that contains the start block and the 100 block file that contains the end block
    let start_100_block = (start_block / 100) * 100;
    let end_100_block = (((end_block as f32) / 100.0).ceil() as usize) * 100;

    let mut blocks: Vec<Block> = Vec::new();
    for block_number in (start_100_block..end_100_block).step_by(100) {
        let block_file_name = directory.to_owned() + &format!("/{:010}.dbin", block_number);
        let block = &decode_flat_files(block_file_name, None, None)
            .map_err(|_| EraValidateError::FlatFileDecodeError)?;
        blocks.extend(block.clone());
    }

    // Return only the requested blocks
    Ok(blocks[start_block - start_100_block..end_block - start_100_block].to_vec())
}
