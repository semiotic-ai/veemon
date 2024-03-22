use std::env;

use clap::{Parser, Subcommand};

use flat_head::era_verifier::verify_eras;

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

        // epoch to start from.
        #[clap(short, long, default_value = "0")]
        start_epoch: usize,

        // epoch to end in. The interval is inclusive
        #[clap(short, long, default_value = "0")]
        end_epoch: Option<usize>,

        #[clap(short = 'c', long, default_value = "true")]
        // Where to decompress files from zstd or not.
        decompress: Option<bool>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.debug {
        0 => {}
        1 => env::set_var("RUST_LOG", "info"),
        2 => env::set_var("RUST_LOG", "debug"),
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
        }) => {
            println!(
                "Starting era validation {} - {}",
                start_epoch,
                end_epoch.map(|x| x.to_string()).unwrap_or("".to_string())
            );

            match verify_eras(
                store_url,
                master_acc_file.as_ref(),
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

    // Continued program logic goes here...
}
