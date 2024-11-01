use ethportal_api::Header;
use flat_files_decoder::decoder::decode_flat_files;

#[test]
fn test_header_from_block() {
    let blocks = decode_flat_files(
        "tests/ethereum_firehose_first_8200/0000008200.dbin".to_string(),
        None,
        None,
        false.into(),
    )
    .unwrap();

    let header = Header::try_from(&blocks[0].clone()).unwrap();
    assert_eq!(header.hash().as_slice(), blocks[0].hash)
}
