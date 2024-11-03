use std::{
    fs::{self, DirEntry, File},
    io::{self, BufReader, BufWriter, Write},
    process::ExitCode,
};

use clap::{Parser, Subcommand};
use firehose_protos::ethereum_v2::Block;
use flat_files_decoder::{
    dbin,
    decoder::{
        handle_reader, stream_blocks, BlockHeaderRoots, Compression, HeaderRecordWithNumber, Reader,
    },
    error::DecoderError,
};
use futures::StreamExt;
use tracing::{error, info, level_filters::LevelFilter, subscriber::set_global_default, trace};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();
    if let Err(e) = run().await {
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
        compression: Compression,
    },
    /// Stream data continuously
    Stream {
        /// decompress .dbin files if they are compressed with zstd
        #[clap(short, long, default_value = "false")]
        compression: Compression,
        /// the block to end streaming
        #[clap(short, long)]
        end_block: Option<u64>,
    },
}

async fn run() -> Result<(), DecoderError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Stream {
            compression,
            end_block,
        } => {
            let mut stream = stream_blocks(Reader::StdIn(compression), end_block.into()).await?;

            let mut writer = BufWriter::new(io::stdout().lock());

            while let Some(block) = stream.next().await {
                let header_record_with_number = HeaderRecordWithNumber::try_from(&block)?;
                let header_record_bin = bincode::serialize(&header_record_with_number)?;

                let size = header_record_bin.len() as u32;
                writer.write_all(&size.to_be_bytes())?;
                writer.write_all(&header_record_bin)?;
                writer.flush()?;
            }

            Ok(())
        }
        Commands::Decode {
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
/// * `input`: A [`String`] specifying the path to the input directory or file.
/// * `output`: An [`Option<&str>`] specifying the directory where decoded blocks should be written.
///             If `None`, decoded blocks are not written to disk.
/// * `headers_dir`: An [`Option<&str>`] specifying the directory containing header files for verification.
///                  Must be a directory if provided.
/// * `compression`: A [`Compression`] enum specifying if it is necessary to decompress from zstd.
fn decode_flat_files(
    input_path: &str,
    output_path: Option<&str>,
    json_headers_dir: Option<&str>,
    compression: Compression,
) -> Result<Vec<Block>, DecoderError> {
    let metadata = fs::metadata(input_path)?;

    // Get blocks depending on file or folder
    let blocks = if metadata.is_dir() {
        info!("Processing directory: {}", input_path);
        read_flat_files(input_path, compression)
    } else {
        info!("Processing file: {}", input_path);
        read_flat_file(input_path, compression)
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

fn write_block_to_json(block: &Block, output: &str) -> Result<(), DecoderError> {
    let file_name = format!("{}/block-{}.json", output, block.number);
    let mut out_file = File::create(file_name)?;

    let block_json = serde_json::to_string(&block)?;

    out_file.write_all(block_json.as_bytes())?;

    Ok(())
}

/// Decodes and verifies block flat files from a single file.
///
/// This function decodes and verifies blocks contained within flat files.
/// Additionally, the function supports handling `zstd` compressed flat files if decompression is required.
///
/// # Arguments
///
/// * `input`: A [`str`] reference specifying the path to the file.
/// * `compression`: A [`Compression`] enum indicating whether decompression from `zstd` format is necessary.
///
fn read_flat_file(path: &str, compression: Compression) -> Result<Vec<Block>, DecoderError> {
    let reader = BufReader::new(File::open(path)?);

    let blocks = handle_reader(reader, compression)?;

    Ok(blocks)
}

fn read_flat_files(path: &str, compression: Compression) -> Result<Vec<Block>, DecoderError> {
    let read_dir = create_read_dir(path)?;

    let mut blocks: Vec<Block> = vec![];

    for path in read_dir {
        let path = path?;

        if file_extension_is_dbin(&path) {
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

fn file_extension_is_dbin(entry: &DirEntry) -> bool {
    dbin::file_extension_is_dbin(entry.path().extension())
}
