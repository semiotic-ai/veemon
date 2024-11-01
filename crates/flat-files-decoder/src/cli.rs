use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use firehose_protos::ethereum_v2::Block;
use tracing::info;

use crate::{
    decoder::{read_flat_file, read_flat_files_dir, stream_blocks, BlockHeaderRoots},
    decompression::Decompression,
    error::DecoderError,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Stream data continuously
    Stream {
        /// decompress .dbin files if they are compressed with zstd
        #[clap(short, long, default_value = "false")]
        decompression: Decompression,
        /// the block to end streaming
        #[clap(short, long)]
        end_block: Option<usize>,
    },
    /// Decode files from input to output
    Decode {
        /// input folder where flat files are stored
        #[clap(short, long)]
        input: String,
        #[clap(long)]
        /// folder where valid headers are stored so decoded blocks can be validated against
        /// their headers.
        headers_dir: Option<String>,
        /// output folder where decoded headers will be stored as .json
        #[clap(short, long)]
        output: Option<String>,
        #[clap(short, long)]
        /// optionally decompress zstd compressed flat files
        decompression: Decompression,
    },
}

pub async fn run() -> Result<(), DecoderError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Stream {
            decompression,
            end_block,
        } => match decompression {
            Decompression::Zstd => {
                let reader = zstd::stream::Decoder::new(io::stdin())?;
                let writer = BufWriter::new(io::stdout().lock());
                stream_blocks(reader, writer, end_block).await
            }
            Decompression::None => {
                let reader = BufReader::with_capacity((64 * 2) << 20, io::stdin().lock());
                let writer = BufWriter::new(io::stdout().lock());
                stream_blocks(reader, writer, end_block).await
            }
        },
        Commands::Decode {
            input,
            headers_dir,
            output,
            decompression,
        } => {
            let blocks = decode_flat_files(
                input,
                output.as_deref(),
                headers_dir.as_deref(),
                decompression,
            )?;

            info!("Total blocks: {}", blocks.len());

            Ok(())
        }
    }
}

/// Decodes and optionally verifies block flat files from a given directory or single file.
///
/// This function processes input which can be a file or a directory containing multiple `.dbin` files.
/// If `headers_dir` is provided, it verifies the block headers against the files found in this directory.
/// These header files must be in JSON format and named after the block number they represent (e.g., `block-<block number>.json`).
/// it can also handle `zstd` compressed flat files.
///
/// # Arguments
///
/// * `input`: A [`String`] specifying the path to the input directory or file.
/// * `output`: An [`Option<&str>`] specifying the directory where decoded blocks should be written.
///             If `None`, decoded blocks are not written to disk.
/// * `headers_dir`: An [`Option<&str>`] specifying the directory containing header files for verification.
///                  Must be a directory if provided.
/// * `decompress`: A [`Decompression`] enum specifying if it is necessary to decompress from zstd.
pub fn decode_flat_files(
    input_path: String,
    output_path: Option<&str>,
    json_headers_dir: Option<&str>,
    decompress: Decompression,
) -> Result<Vec<Block>, DecoderError> {
    let metadata = fs::metadata(&input_path)?;

    // Get blocks depending on file or folder
    let blocks = if metadata.is_dir() {
        read_flat_files_dir(&input_path, decompress)
    } else {
        read_flat_file(&PathBuf::from(input_path), decompress)
    }?;

    if let Some(json_headers_dir) = json_headers_dir {
        for block in blocks.iter() {
            check_block_against_json(block, json_headers_dir)?;
        }
    }

    if let Some(path) = output_path {
        fs::create_dir_all(path)?;
        for block in blocks.iter() {
            write_block_to_json(block, path)?;
        }
    }

    Ok(blocks)
}

fn check_block_against_json(block: &Block, headers_dir: &str) -> Result<(), DecoderError> {
    let header_file_path = format!("{}/{}.json", headers_dir, block.number);
    let header_file = File::open(header_file_path)?;
    let header_roots: BlockHeaderRoots = serde_json::from_reader(header_file)?;

    if !header_roots.block_header_matches(block) {
        return Err(DecoderError::MatchRootsFailed {
            block_number: block.number,
        });
    }

    Ok(())
}

fn write_block_to_json(block: &Block, output: &str) -> Result<(), DecoderError> {
    let file_name = format!("{}/block-{}.json", output, block.number);
    let mut out_file = File::create(file_name)?;

    let block_json = serde_json::to_string(&block)?;

    out_file.write_all(block_json.as_bytes())?;

    Ok(())
}
