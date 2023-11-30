use decoder::decode_flat_files;
use ethportal_api::types::execution::accumulator::{EpochAccumulator, HeaderRecord};
use primitive_types::{H256 as Hash256, U256};
use tree_hash::TreeHash;
use trin_validation::accumulator::MasterAccumulator;

const MAX_EPOCH_SIZE: usize = 8192;

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

pub fn era_validate(directory: &String, master_accumulator_file: Option<&String>, start_epoch: usize, end_epoch: Option<usize>) {
    let master_accumulator;
    // Load master accumulator if available, otherwise use default from Prortal Network
    if master_accumulator_file.is_some() {
        master_accumulator = MasterAccumulator::try_from_file(master_accumulator_file.unwrap().into()).unwrap();
    } else {
        master_accumulator = MasterAccumulator::default();
    }

    // Load blocks from flat files
    let blocks = decode_flat_files(directory, None, None).unwrap();
    let mut header_records = Vec::<HeaderRecord>::new();
    let start_block_number = start_epoch * MAX_EPOCH_SIZE;
    let end_block_number;
    if let Some(end_epoch) = end_epoch {
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

    println!("SUCCESS: Flat file accumulator matches master accumulator!");
}
