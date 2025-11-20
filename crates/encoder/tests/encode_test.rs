// Copyright 2025-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use flat_files_encoder::{Encoder, FrameKind};

#[test]
fn test_eth_block_encode_to_dbin_header() {
    let enc = Encoder::new_v1("type.googleapis.com/sf.ethereum.type.v1.Block");

    // Encode nothing, just to get the header into a Vec<u8>.
    let mut dbin = Vec::new();
    enc.encode_with(&mut dbin, std::iter::empty::<()>(), FrameKind::Raw, |_| {
        Vec::new()
    })
    .unwrap();

    assert!(dbin.starts_with(b"dbin"));
    assert_eq!(dbin[4], 1);

    let len = u16::from_be_bytes([dbin[5], dbin[6]]);
    let expected = b"type.googleapis.com/sf.ethereum.type.v1.Block";
    assert_eq!(len as usize, expected.len());
    let ct = &dbin[7..7 + (len as usize)];
    assert_eq!(ct, expected);
}

#[test]
fn test_sol_block_encode_to_dbin_header() {
    let enc = Encoder::new_v1("type.googleapis.com/sf.solana.type.v1.Block");

    // Encode nothing, just to get the header into a Vec<u8>.
    let mut dbin = Vec::new();
    enc.encode_with(&mut dbin, std::iter::empty::<()>(), FrameKind::Raw, |_| {
        Vec::new()
    })
    .unwrap();

    assert!(dbin.starts_with(b"dbin"));
    assert_eq!(dbin[4], 1);

    let len = u16::from_be_bytes([dbin[5], dbin[6]]);
    let ct = &dbin[7..7 + (len as usize)];
    let expected = b"type.googleapis.com/sf.solana.type.v1.Block";
    assert_eq!(ct, expected);
}
