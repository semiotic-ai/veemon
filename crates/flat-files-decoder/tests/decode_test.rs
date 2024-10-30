use flat_files_decoder::{decode_flat_files, Decompression};

const BLOCK_NUMBER: usize = 0;

#[test]
fn test_decode_decompressed() {
    let file_name = format!("tests/{:010}.dbin", BLOCK_NUMBER);
    let blocks = decode_flat_files(file_name, None, None, Decompression::None).unwrap();
    assert_eq!(blocks.len(), 100);
}

#[test]
fn test_decode_compressed() {
    let file_name = format!("tests/{:010}.dbin.zst", BLOCK_NUMBER);
    let blocks_compressed = decode_flat_files(file_name, None, None, Decompression::Zstd).unwrap();
    assert_eq!(blocks_compressed.len(), 100);

    let file_name = format!("tests/{:010}.dbin", BLOCK_NUMBER);
    let blocks_decompressed =
        decode_flat_files(file_name, None, None, Decompression::None).unwrap();
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
