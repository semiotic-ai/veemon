use firehose_protos::{bstream::v1::Block as BstreamBlock, ethereum_v2::Block};
use flat_files_decoder::{
    dbin::DbinFile,
    decoder::{handle_buffer, stream_blocks, Compression},
};
use futures::StreamExt;
use prost::Message;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Cursor, Write},
    path::PathBuf,
};

const BLOCK_NUMBER: usize = 0;

const TEST_ASSET_PATH: &str = "../../test-assets";

fn create_test_reader(path: &str) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}

#[test]
fn test_dbin_try_from_read() {
    let mut reader =
        BufReader::new(File::open(format!("{TEST_ASSET_PATH}/example0017686312.dbin")).unwrap());

    let dbin_file = DbinFile::try_from_read(&mut reader).unwrap();

    insta::assert_debug_snapshot!(dbin_file.header.content_type, @r###""ETH""###);
}

#[test]
fn test_decode_decompressed() {
    let file = format!("{TEST_ASSET_PATH}/{:010}.dbin", BLOCK_NUMBER);
    let blocks = handle_buffer(create_test_reader(file.as_str()), Compression::None).unwrap();
    assert_eq!(blocks.len(), 100);
}

#[test]
fn test_decode_compressed() {
    let file = format!("{TEST_ASSET_PATH}/{:010}.dbin.zst", BLOCK_NUMBER);
    let blocks_compressed =
        handle_buffer(create_test_reader(file.as_str()), Compression::Zstd).unwrap();
    assert_eq!(blocks_compressed.len(), 100);

    let file = format!("{TEST_ASSET_PATH}/{:010}.dbin", BLOCK_NUMBER);
    let blocks_decompressed =
        handle_buffer(create_test_reader(file.as_str()), Compression::None).unwrap();
    assert_eq!(blocks_compressed.len(), blocks_decompressed.len());
    for (b1, b2) in blocks_compressed.into_iter().zip(blocks_decompressed) {
        assert_eq!(b1.hash, b2.hash);
        assert_eq!(b1.number, b2.number);
        assert_eq!(b1.size, b2.size);
        assert_eq!(b1.header, b2.header);
        assert_eq!(b1.detail_level, b2.detail_level);
        assert_eq!(b1.uncles, b2.uncles);
        assert_eq!(b1.code_changes, b2.code_changes);
        assert_eq!(b1.balance_changes, b2.balance_changes);
        assert_eq!(b1.transaction_traces, b2.transaction_traces);
        assert_eq!(b1.system_calls, b2.system_calls);
    }
}

#[test]
fn test_handle_file() {
    let file = format!("{TEST_ASSET_PATH}/example0017686312.dbin");
    let result = handle_buffer(create_test_reader(file.as_str()), Compression::None);
    assert!(result.is_ok());
}

#[test]
fn test_handle_file_zstd() {
    let file = format!("{TEST_ASSET_PATH}/0000000000.dbin.zst");
    let result = handle_buffer(create_test_reader(file.as_str()), Compression::Zstd);
    assert!(result.is_ok());
    let blocks: Vec<Block> = result.unwrap();
    assert_eq!(blocks[0].number, 0);
}

#[test]
fn test_check_valid_root_fail() {
    let path = PathBuf::from(format!("{TEST_ASSET_PATH}/example0017686312.dbin"));
    let mut file = BufReader::new(File::open(path).expect("Failed to open file"));
    let dbin_file: DbinFile =
        DbinFile::try_from_read(&mut file).expect("Failed to parse dbin file");

    let message = dbin_file.messages[0].clone();

    let block_stream = BstreamBlock::decode(message.as_slice()).unwrap();
    let mut block = Block::decode(block_stream.payload_buffer.as_slice()).unwrap();

    assert!(block.receipt_root_is_verified());

    // Remove an item from the block to make the receipt root invalid
    block.transaction_traces.pop();

    assert!(!block.receipt_root_is_verified());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_block_stream() {
    let mut buffer = Vec::new();
    let cursor: Cursor<&mut Vec<u8>> = Cursor::new(&mut buffer);
    let inputs = vec![
        format!("{TEST_ASSET_PATH}/example-create-17686085.dbin"),
        format!("{TEST_ASSET_PATH}/example0017686312.dbin"),
    ];
    {
        let mut writer = BufWriter::new(cursor);
        for i in inputs {
            let mut input = File::open(i).expect("couldn't read input file");

            std::io::copy(&mut input, &mut writer).expect("couldn't copy");
            writer.flush().expect("failed to flush output");
        }
    }
    let mut cursor = Cursor::new(&buffer);
    cursor.set_position(0);

    let reader = BufReader::new(cursor);

    let mut blocks = Vec::new();

    let mut stream = stream_blocks(reader, None).await.unwrap();

    while let Some(block) = stream.next().await {
        blocks.push(block);
    }

    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].number, 17686164);
    assert_eq!(blocks[1].number, 17686312);
}

#[test]
fn test_handle_buffer() {
    let path = PathBuf::from(format!("{TEST_ASSET_PATH}/example0017686312.dbin"));
    let file = BufReader::new(File::open(path).expect("Failed to open file"));
    let result = handle_buffer(file, false.into());
    if let Err(e) = result {
        panic!("handle_buf failed: {}", e);
    }
    assert!(result.is_ok(), "handle_buf should complete successfully");
}

#[test]
fn test_handle_buffer_decompress() {
    let file = format!("{TEST_ASSET_PATH}/0000000000.dbin.zst");
    let result = handle_buffer(create_test_reader(file.as_str()), Compression::Zstd);
    assert!(
        result.is_ok(),
        "handle_buf should complete successfully with decompression"
    );
}
