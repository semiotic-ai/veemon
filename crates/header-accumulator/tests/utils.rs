use decoder::decode_flat_files;
use ethportal_api::Header;

#[test]
fn test_header_from_block() {
    let blocks = decode_flat_files(
        "tests/ethereum_firehose_first_8200/0000008200.dbin".to_string(),
        None,
        None,
        Some(false),
    )
    .unwrap();

    let header = Header::try_from(&blocks[0].clone()).unwrap();
    assert_eq!(header.hash().as_slice(), blocks[0].hash)
}
