// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::{self, DirEntry, File}, io::{self, BufReader, BufWriter, Write}, process::ExitCode
};

use alloy_primitives::B256;
use clap::{Parser, Subcommand};
use firehose_protos::{BlockHeader, EthBlock as Block, SolBlock};
use flat_files_decoder::{
    read_blocks_from_reader, stream_blocks, AnyBlock, Compression, DecoderError, Reader,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, level_filters::LevelFilter, subscriber::set_global_default, trace};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

fn main() -> ExitCode {
    init_tracing();
    if let Err(e) = run() {
        error!("Decoder error: {e}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn init_tracing() {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    let subscriber_builder: tracing_subscriber::fmt::SubscriberBuilder<
        tracing_subscriber::fmt::format::DefaultFields,
        tracing_subscriber::fmt::format::Format,
        EnvFilter,
    > = FmtSubscriber::builder().with_env_filter(filter);
    set_global_default(subscriber_builder.with_ansi(true).pretty().finish()).expect(
        "Failed to set up the global default subscriber for logging. Please check if the RUST_LOG environment variable is set correctly.",
    );
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Decodes files from an input folder and can save them to an output folder
    Decode {
        /// Path to the input folder containing flat files
        #[clap(short, long)]
        input: String,

        /// Optional path to a folder containing headers for validating decoded blocks
        #[clap(long)]
        headers_dir: Option<String>,

        /// Optional path to an output folder for saving decoded headers as .json files
        #[clap(short, long)]
        output: Option<String>,

        /// Enables decompression for zstd-compressed flat files
        #[clap(short, long, default_value = "false")]
        compression: Compression,
    },

    /// Stream data continuously
    Stream {
        /// Decompresses .dbin files if they are zstd-compressed
        #[clap(short, long, default_value = "false")]
        compression: Compression,

        /// Block number to end the streaming process
        #[clap(short, long)]
        end_block: Option<u64>,
    },
}

fn run() -> Result<(), DecoderError> {
    use Commands::*;

    let cli = Cli::parse();

    match cli.command {
        Stream {
            compression,
            end_block,
        } => {
            let blocks = stream_blocks(Reader::StdIn(compression), end_block.into())?;

            let mut writer = BufWriter::new(io::stdout().lock());

            for block in blocks {
                let header_record_with_number = HeaderRecordWithNumber::try_from(&block)?;
                let header_record_bin = bincode::serialize(&header_record_with_number)?;

                let size = header_record_bin.len() as u32;
                writer.write_all(&size.to_be_bytes())?;
                writer.write_all(&header_record_bin)?;
                writer.flush()?;
            }

            Ok(())
        }
        Decode {
            input,
            headers_dir,
            output,
            compression,
        } => {
            let blocks = decode_flat_files(
                &input,
                output.as_deref(),
                headers_dir.as_deref(),
                compression,
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
/// * `input_path`: A [`String`] specifying the path to the input directory or file.
/// * `output_path`: An [`Option<&str>`] specifying the directory where decoded blocks should be written.
///             If `None`, decoded blocks are not written to disk.
/// * `json_headers_dir`: An [`Option<&str>`] specifying the directory containing EVM Block Header files for verification.
///                  Must be a directory if provided.
/// * `compression`: A [`Compression`] enum specifying if it is necessary to decompress from zstd.
fn decode_flat_files(
    input_path: &str,
    output_path: Option<&str>,
    json_headers_dir: Option<&str>,
    compression: Compression,
) -> Result<Vec<AnyBlock>, DecoderError> {
    let metadata = fs::metadata(input_path)?;

    // Get blocks depending on file or folder
    let blocks = if metadata.is_dir() {
        info!("Processing directory: {}", input_path);
        read_flat_files(input_path, compression)
    } else {
        info!("Processing file: {}", input_path);
        read_flat_file(input_path, compression)
    }?;

    // These JSON file formats are applicable to EVM Block Headers.
    if let Some(json_headers_dir) = json_headers_dir {
        for block in blocks.iter() {
            match block {
                AnyBlock::Eth(eth_block) => {
                    check_block_against_json(eth_block, json_headers_dir)?;
                }
                _ => {
                    info!("JSON Headers Directory provided, but no EVM block found.");
                    break;
                }
            }
            
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

fn create_read_dir(input_path: &str) -> io::Result<fs::ReadDir> {
    fs::read_dir(input_path)
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

fn write_block_to_json(block: &AnyBlock, output: &str) -> Result<(), DecoderError> {
    let block_number = match block {
        AnyBlock::Eth(eth_block) => {
            eth_block.number
        }
        AnyBlock::Sol(sol_block) => {
            sol_block.block_height.unwrap().block_height
        }
    };

    let file_name = format!("{}/block-{}.json", output, block_number);
    let mut out_file = File::create(file_name)?;

    let block_json = serde_json::to_string(&block)?;

    out_file.write_all(block_json.as_bytes())?;

    Ok(())
}

/// Decodes and verifies block flat files from a single file.
///
/// This function decodes and verifies blocks contained within flat files.
/// Additionally, the function supports handling `zstd` compressed flat files if decompression is required.
fn read_flat_file(path: &str, compression: Compression) -> Result<Vec<AnyBlock>, DecoderError> {
    let reader = BufReader::new(File::open(path)?);

    let blocks = read_blocks_from_reader(reader, compression)?;

    Ok(blocks)
}

/// Dbin file type extension
const EXTENSION: &str = "dbin";

/// Checks if the file extension is `.dbin`.
fn dir_entry_extension_is_dbin(entry: &DirEntry) -> bool {
    entry
        .path()
        .extension()
        .map_or(false, |ext| ext == EXTENSION)
}

fn read_flat_files(path: &str, compression: Compression) -> Result<Vec<AnyBlock>, DecoderError> {
    let read_dir = create_read_dir(path)?;

    let mut blocks: Vec<AnyBlock> = vec![];

    for path in read_dir {
        let path = path?;

        if !dir_entry_extension_is_dbin(&path) {
            continue;
        }

        trace!("Processing file: {}", path.path().display());

        match read_flat_file(path.path().to_str().unwrap(), compression) {
            Ok(blocks_vec) => {
                blocks.extend(blocks_vec);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(blocks)
}

/// A struct to hold the block hash, block number, and total difficulty of a block.
#[derive(Serialize, Deserialize)]
struct HeaderRecordWithNumber {
    block_hash: Vec<u8>,
    block_number: u64,
    total_difficulty: Vec<u8>,
}

/// Try from an Ethereum Block
impl TryFrom<&Block> for HeaderRecordWithNumber {
    type Error = DecoderError;

    fn try_from(block: &Block) -> Result<Self, Self::Error> {
        Ok(HeaderRecordWithNumber {
            block_hash: block.hash.clone(),
            block_number: block.number,
            total_difficulty: block
                .header()?
                .total_difficulty
                .as_ref()
                .ok_or(Self::Error::TotalDifficultyInvalid)?
                .bytes
                .clone(),
        })
    }
}

/// Try from a Solana Block
impl TryFrom<&SolBlock> for HeaderRecordWithNumber {
    type Error = DecoderError;

    fn try_from(block: &SolBlock) -> Result<Self, Self::Error> {
        Ok(HeaderRecordWithNumber {
            block_hash: block.blockhash.clone().into(),
            block_number: block.block_height.unwrap().block_height, 
            // There is no field analogous to `total_difficulty` in Solana Blocks
            total_difficulty: Vec::new(),
        })
    }
}

/// Try from a Generalized AnyBlock enum
impl TryFrom<&AnyBlock> for HeaderRecordWithNumber {
    type Error = DecoderError;

    fn try_from(block: &AnyBlock) -> Result<Self, Self::Error> {
        match block {
            AnyBlock::Eth(eth_block) => {
                HeaderRecordWithNumber::try_from(eth_block)
            }
            AnyBlock::Sol(sol_block) => {
                HeaderRecordWithNumber::try_from(sol_block)
            }
        }
    }
}

/// A struct to hold the receipt and transactions root for an [`Block`].
/// This struct is used to compare the receipt and transactions roots of a block
/// with the receipt and transactions roots of another block.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct BlockHeaderRoots {
    receipt_root: B256,
    transactions_root: B256,
}

impl TryFrom<&Block> for BlockHeaderRoots {
    type Error = DecoderError;

    fn try_from(block: &Block) -> Result<Self, Self::Error> {
        block.header()?.try_into()
    }
}

impl TryFrom<&BlockHeader> for BlockHeaderRoots {
    type Error = DecoderError;

    fn try_from(header: &BlockHeader) -> Result<Self, Self::Error> {
        let receipt_root: [u8; 32] = header.receipt_root.as_slice().try_into()?;
        let transactions_root: [u8; 32] = header.transactions_root.as_slice().try_into()?;

        Ok(Self {
            receipt_root: receipt_root.into(),
            transactions_root: transactions_root.into(),
        })
    }
}

impl BlockHeaderRoots {
    /// Checks if the receipt and transactions roots of a block header match the receipt and transactions roots of another block.
    fn block_header_matches(&self, block: &Block) -> bool {
        match block.try_into() {
            Ok(other) => self == &other,
            Err(e) => {
                error!("Failed to convert block to header roots: {e}");
                false
            }
        }
    }
}
