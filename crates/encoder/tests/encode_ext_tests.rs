use firehose_protos::{EthBlock, SolBlock};
use flat_files_encoder::DbinEncodeExt; // trait in this crate

#[test]
fn test_eth_block_encode_to_dbin_header() {
    let eth = EthBlock::default();
    let dbin = eth.encode_to_dbin();

    assert!(dbin.starts_with(b"dbin"));
    assert_eq!(dbin[4], 1); // Version::V1

    let len = u16::from_be_bytes([dbin[5], dbin[6]]);
    assert_eq!(len, 3);
    assert_eq!(&dbin[7..10], b"ETH");
}

#[test]
fn test_sol_block_encode_to_dbin_header() {
    let sol = SolBlock::default();
    let dbin = sol.encode_to_dbin();

    assert!(dbin.starts_with(b"dbin"));
    assert_eq!(dbin[4], 1);

    let len = u16::from_be_bytes([dbin[5], dbin[6]]);
    let ct = &dbin[7..7 + (len as usize)];
    let expected = b"type.googleapis.com/sf.solana.type.v1.Block";
    assert_eq!(ct, expected);
}
