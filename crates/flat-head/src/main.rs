use std::env;

use clap::{Parser, Subcommand};

use flat_head::era_verifier::verify_eras;
use trin_validation::accumulator::MasterAccumulator;

#[derive(Parser)]
#[command(version, about = "A flat file decoder and validator", long_about = None)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count, help = "Increase debug level (use -d for debug, -dd for trace, etc.)")]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

// a flat file decoder and validator
#[derive(Subcommand)]
enum Commands {
    /// Decode and validates flat files from a directory.
    EraValidate {
        #[clap(short = 'b', long)]
        // directory where flat files are located
        store_url: String,

        #[clap(short, long)]
        // master accumulator file. default Portal Network file will be used if none provided
        master_acc_file: Option<String>,

        #[clap(short, long, default_value = "0")]
        // epoch to start from.
        start_epoch: usize,

        #[clap(short, long, default_value = "0")]
        // epoch to end in. The interval is inclusive
        end_epoch: Option<usize>,

        #[clap(short = 'c', long, default_value = "true")]
        // Where to decompress files from zstd or not.
        decompress: Option<bool>,

        #[clap(short = 'p', long)]
        // indicates if the store_url is compatible with some API. E.g., if `--compatible s3` is used,
        // then the store_url can point to seaweed-fs with S3 compatibility enabled and work as intended.
        compatible: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.debug {
        0 => env::set_var("RUST_LOG", "info"),
        1 => env::set_var("RUST_LOG", "debug"),
        _ => {}
    }
    env_logger::init();

    match &cli.command {
        Some(Commands::EraValidate {
            decompress,
            store_url,
            master_acc_file,
            start_epoch,
            end_epoch,
            compatible,
        }) => {
            println!(
                "Starting era validation {} - {}",
                start_epoch,
                end_epoch.map(|x| x.to_string()).unwrap_or("".to_string())
            );

            let macc = match master_acc_file {
                Some(master_accumulator_file) => {
                    MasterAccumulator::try_from_file(master_accumulator_file.into())
                        .map_err(|_| panic!("failed to parse master accumulator file"))
                }
                None => Ok(MasterAccumulator::default()),
            };

            match verify_eras(
                store_url.to_string(),
                macc.unwrap(),
                compatible.clone(),
                *start_epoch,
                *end_epoch,
                *decompress,
            )
            .await
            {
                Ok(result) => {
                    println!("Epochs validated: {:?}", result);
                }
                Err(e) => {
                    log::error!("error: {:#}", e);
                }
            }
        }
        None => {}
    }
}
