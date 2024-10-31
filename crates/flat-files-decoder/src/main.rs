use clap::{Parser, Subcommand};
use flat_files_decoder::{
    decode_flat_files, decompression::Decompression, error::DecoderError, stream_blocks,
};
use std::{
    io::{self, BufReader, BufWriter},
    process::ExitCode,
};
use tracing::{error, info, level_filters::LevelFilter, subscriber::set_global_default};
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

async fn run() -> Result<(), DecoderError> {
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

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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
