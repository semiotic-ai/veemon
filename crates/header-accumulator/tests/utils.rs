use decoder::decode_flat_files;
use header_accumulator::utils::header_from_block;

#[test]
fn test_header_from_block() {
    let blocks = decode_flat_files(
        "tests/ethereum_firehose_first_8200/0000008200.dbin".to_string(),
        None,
        None,
    )
    .unwrap();

    let header = header_from_block(blocks[0].clone()).unwrap();
    assert_eq!(header.hash().as_bytes(), blocks[0].hash)
}
