// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{fs::File, io::BufReader};

use flat_files_decoder::{read_blocks_from_reader, Compression};
use header_accumulator::{Epoch, EraValidateError, EraValidator, ExtHeaderRecord};
use tree_hash::Hash256;
use trin_validation::accumulator::PreMergeAccumulator;

fn create_test_reader(path: &str) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}

fn main() -> Result<(), EraValidateError> {
    let mut headers: Vec<ExtHeaderRecord> = Vec::new();

    for number in (0..=8200).step_by(100) {
        let file_name = format!(
            "your-test-assets/ethereum_firehose_first_8200/{:010}.dbin",
            number
        );
        let reader = create_test_reader(&file_name);
        let blocks = read_blocks_from_reader(reader, Compression::None).unwrap();
        let successful_headers = blocks
            .iter()
            .cloned()
            .map(|block| ExtHeaderRecord::try_from(&block))
            .collect::<Result<Vec<_>, _>>()?;
        headers.extend(successful_headers);
    }

    assert_eq!(headers.len(), 8300);
    assert_eq!(headers[0].block_number, 0);

    let premerge_accumulator: EraValidator = PreMergeAccumulator::default().into();
    let epoch: Epoch = headers.try_into().unwrap();

    let result = premerge_accumulator.validate_era(&epoch)?;
    let expected = Hash256::new([
        94, 193, 255, 184, 195, 177, 70, 244, 38, 6, 199, 76, 237, 151, 61, 193, 110, 197, 161, 7,
        192, 52, 88, 88, 195, 67, 252, 148, 120, 11, 66, 24,
    ]);
    assert_eq!(result, expected);

    println!("Era validated successfully!");

    Ok(())
}
