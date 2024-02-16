use clap::{Parser, Subcommand};
use decoder::{decode_flat_files, stream_blocks};
use std::io::{self, BufReader, BufWriter};

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
        /// decompress .dibn files if they are compressed with zstd
        #[clap(short, long, default_value = "false")]
        decompress: bool,
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
    },
}
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Stream {
            decompress,
            end_block,
        } => {
            if decompress {
                // zst decompress first
                let reader =
                    zstd::stream::Decoder::new(io::stdin()).expect("Failed to create zstd decoder");
                let writer = BufWriter::new(io::stdout().lock());
                stream_blocks(reader, writer, end_block)
                    .await
                    .expect("Failed to stream blocks");
            } else {
                let reader = BufReader::with_capacity((64 * 2) << 20, io::stdin().lock());
                let writer = BufWriter::new(io::stdout().lock());
                stream_blocks(reader, writer, end_block)
                    .await
                    .expect("Failed to stream blocks");
            }
        }
        Commands::Decode {
            input,
            headers_dir,
            output,
        } => {
            let blocks = decode_flat_files(input, output.as_deref(), headers_dir.as_deref())
                .expect("Failed to decode files");

            println!("Total blocks: {}", blocks.len());
        }
    }
}
