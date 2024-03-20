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
        #[clap(short, long)]
        // directory where flat files are located
        dir: String,

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
        // directory where flat files are located
        decompress: Option<bool>,
    },
}

fn main() {
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
            dir,
            master_acc_file,
            start_epoch,
            end_epoch,
        }) => {
            let result = verify_eras(
                dir,
                master_acc_file.as_ref(),
                *start_epoch,
                *end_epoch,
                *decompress,
            );
            log::info!("epochs validated: {:?}", result);
        }
        None => {}
    }

    // Continued program logic goes here...
}
