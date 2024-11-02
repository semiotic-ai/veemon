use std::{fs::File, io::BufReader};

use flat_files_decoder::{compression::Compression, decoder::handle_buffer};
use header_accumulator::{Epoch, EraValidateError, EraValidator, ExtHeaderRecord};
use tree_hash::Hash256;
use trin_validation::accumulator::PreMergeAccumulator;

fn create_test_reader(path: &str) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}

#[test]
fn test_era_validate() -> Result<(), EraValidateError> {
    let mut headers: Vec<ExtHeaderRecord> = Vec::new();
    for number in (0..=8200).step_by(100) {
        let file_name = format!("tests/ethereum_firehose_first_8200/{:010}.dbin", number);
        let reader = create_test_reader(&file_name);
        let blocks = handle_buffer(reader, Compression::None).unwrap();
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

    Ok(())
}
