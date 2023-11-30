use clap::{Arg, Command};
use era_validator::era_validate;
use std::process;

mod era_validator;

fn main() {
    let matches = Command::new("header_accumulator")
        .version("0")
        .author("Semiotic Labs")
        .about("Validates flat files against Header Accumulators")
        .subcommand(
            Command::new("era_validate")
                .about("Validates entire ERAs of flat files against Header Accumulators")
                .arg(Arg::new("directory")
                    .help("Directory where the flat files are stored")
                    .required(true)
                    .index(1))
                .arg(Arg::new("start_epoch")
                    .help("Start epoch to check")
                    .required(false)
                    .short('s')
                    .long("start_epoch"))
                .arg(Arg::new("end_epoch")
                    .help("End epoch to check")
                    .required(false)
                    .short('e')
                    .long("end_epoch"))
                .arg(Arg::new("master_accumulator_file")
                    .help("Master accumulator file (optional)")
                    .required(false)
                    .short('m')
                    .long("master_accumulator_file"))
        )
        .get_matches();

    match matches.subcommand() {
        Some(("era_validate", era_validate_matches)) => {
            let directory = era_validate_matches.get_one::<String>("directory").expect("Directory is required.");
            let master_accumulator_file = era_validate_matches.get_one::<String>("master_accumulator_file");
            let start_epoch = era_validate_matches.get_one::<String>("start_epoch");
            let end_epoch = era_validate_matches.get_one::<String>("end_epoch");

            let start_epoch = match start_epoch {
                Some(start_epoch) => start_epoch.parse::<usize>().unwrap(),
                None => 0,
            };

            let end_epoch = match end_epoch {
                Some(end_epoch) => Some(end_epoch.parse::<usize>().unwrap()),
                None => None,
            };

            if let Err(result) = era_validate(directory, master_accumulator_file, start_epoch, end_epoch){
                println!("Error: {}", result);
                process::exit(1);
            }
            process::exit(0);
        }
        _ => {
            println!("No subcommand was used");
        }
    }
}
