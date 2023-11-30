use clap::Parser;
use decoder::decode_flat_files;
use ethportal_api::types::execution::accumulator::{EpochAccumulator, HeaderRecord};
use primitive_types::{H256 as Hash256, U256};
use tree_hash::TreeHash;
use trin_validation::accumulator::MasterAccumulator;

const MAX_EPOCH_SIZE: usize = 8192;

/// A program to check whether the validity of block headers stored in a directory of flat files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory where the flat files are stored
    #[arg(short, long)]
    directory: String,

    /// Master accumulator file
    #[arg(short, long, default_value = None)]
    master_accumulator_file: Option<String>,

    // Start epoch to check
    #[arg(short, long, default_value = "0")]
    start_epoch: usize,

    // End epoch to check
    #[arg(short, long, default_value = None)]
    end_epoch: Option<usize>,
}

fn compute_epoch_accumulator(header_records: Vec<HeaderRecord>) -> EpochAccumulator {
    if header_records.len() > MAX_EPOCH_SIZE {
        panic!("hashes and tds must be less than MAX_EPOCH_SIZE");
    }

    let mut epoch_accumulator = EpochAccumulator::new(Vec::new()).unwrap();
    for header_record in header_records {
        let _ = epoch_accumulator.push(header_record);
    }
    epoch_accumulator
}

fn main() {
    let args = Args::parse();
    let master_accumulator;

    // Load master accumulator if available, otherwise use default from Prortal Network
    if args.master_accumulator_file.is_some() {
        let master_accumulator_file = args.master_accumulator_file.unwrap();
        master_accumulator =
            MasterAccumulator::try_from_file(master_accumulator_file.into()).unwrap();
    } else {
        master_accumulator = MasterAccumulator::default();
    }

    // Load blocks from flat files
    let blocks = decode_flat_files(&args.directory, None, None).unwrap();

    let mut header_records = Vec::<HeaderRecord>::new();
    let start_block_number = args.start_epoch * MAX_EPOCH_SIZE;
    let end_block_number;
    if let Some(end_epoch) = args.end_epoch {
        end_block_number = end_epoch * MAX_EPOCH_SIZE;
        assert!(end_block_number <= blocks.len());
    }
    else {
        end_block_number = start_block_number + MAX_EPOCH_SIZE;
    }
    let mut block_number = start_block_number;
    while block_number < end_block_number {
        let block = blocks
            .iter()
            .find(|&b| b.number == block_number as u64)
            .unwrap();
        let header_record = HeaderRecord {
            block_hash: Hash256::from_slice(block.hash.as_slice()),
            total_difficulty: U256::try_from(
                block
                    .header
                    .total_difficulty
                    .as_ref()
                    .unwrap()
                    .bytes
                    .as_slice(),
            )
            .unwrap(),
        };
        header_records.push(header_record);
        block_number += 1;
    }

    let epoch_accumulator = compute_epoch_accumulator(header_records);

    assert_eq!(
        epoch_accumulator.tree_hash_root().0,
        master_accumulator.historical_epochs[0].0
    );

    println!("Flat file accumulator matches master accumulator!");
}
