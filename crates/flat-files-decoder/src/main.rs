use clap::Parser;
use flat_files_decoder::{
    cli::{Cli, Commands},
    decode_flat_files,
    decompression::Decompression,
    error::DecoderError,
    stream_blocks,
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
