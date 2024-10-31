use clap::{Parser, Subcommand};

use crate::decompression::Decompression;

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
