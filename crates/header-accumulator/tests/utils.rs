use std::{fs::File, io::BufReader};

use ethportal_api::Header;
use flat_files_decoder::decoder::{handle_buffer, Compression};

fn create_test_reader(path: &str) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}

#[test]
fn test_header_from_block() {
    let blocks = handle_buffer(
        create_test_reader("tests/ethereum_firehose_first_8200/0000008200.dbin"),
        Compression::None,
    )
    .unwrap();

    let header = Header::try_from(&blocks[0].clone()).unwrap();
    assert_eq!(header.hash().as_slice(), blocks[0].hash)
}
