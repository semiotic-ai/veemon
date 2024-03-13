use std::env;

use clap::{Parser, Subcommand};

use flat_head::era_verifier::verify_eras;

#[derive(Parser)]
#[command(version, about = "A flat file decoder and validator", long_about = None)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    // debug values in format -d, -dd, -ddd...
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

// a flat file decoder and validator
#[derive(Subcommand)]
enum Commands {
    /// Decode files from input to output
    EraValidate {
        #[clap(short, long)]
        // directory where flat files are located
        dir: String,

        #[clap(long)]
        // #[clap(short, long)]
        // // directory where valid blocks will be stored in
        // output: Option<String>,
        #[clap(short, long)]
        // master accumulator file. default Portal Network file will be used if none provided
        master_acc_file: Option<String>,

        #[clap(short, long, default_value = "0")]
        start_epoch: usize,

        #[clap(short, long, default_value = "0")]
        end_epoch: Option<usize>,
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
            dir,
            master_acc_file,
            start_epoch,
            end_epoch,
        }) => {
            // if *list {
            //     println!("Printing testing lists...");
            // }
            {
                log::info!("Starting validation.");
                let result = verify_eras(dir, master_acc_file.as_ref(), *start_epoch, *end_epoch);
                log::info!("epochs validated: {:?}", result);
            }
        }
        None => {}
    }

    // Continued program logic goes here...
}
