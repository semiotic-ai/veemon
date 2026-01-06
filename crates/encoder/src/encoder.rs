// SPDX-FileCopyrightText: 2025- Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

// Encoder crate — DBIN-aligned encoder.
//
// This crate encodes raw block data into a DBIN-like binary stream that can be
// consumed by the decoder in `crates/decoder`.

use std::io::{self, Write};

use firehose_protos::BstreamBlock;
use prost::Message;
use tracing::warn;

/// Encoder configuration; stores only fields needed for each version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncoderConfig {
    /// DBIN v0: fixed 3-byte content type and 2-byte version.
    V0 {
        /// Three-byte ASCII content type (e.g., "ETH").
        content_type: [u8; 3],
        /// Two-byte content version for v0 headers.
        content_version: [u8; 2],
    },
    /// DBIN v1: arbitrary-length UTF-8 content type string.
    V1 {
        /// Arbitrary content type identifier (e.g., type URL or short code).
        content_type: String,
    },
}

impl EncoderConfig {
    fn content_type_str(&self) -> io::Result<&str> {
        match self {
            EncoderConfig::V0 { content_type, .. } => {
                core::str::from_utf8(content_type).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "V0 content type must be valid UTF-8 for Bstream frames",
                    )
                })
            }
            EncoderConfig::V1 { content_type } => Ok(content_type.as_str()),
        }
    }
}

/// Frame encoding mode.
#[derive(Debug, Clone, Copy)]
pub enum FrameKind {
    /// Frame bytes are written directly.
    Raw,
    /// Frame bytes are wrapped in a [`BstreamBlock`] protobuf.
    Bstream,
}

const ETH_HEADER: &str = "type.googleapis.com/sf.ethereum.type.v2.Block";
const SOLANA_HEADER: &str = "type.googleapis.com/sf.solana.type.v1.Block";

fn type_url_for(content_type: &str) -> &str {
    match content_type {
        "ETH" | ETH_HEADER => ETH_HEADER,
        SOLANA_HEADER => SOLANA_HEADER,
        _ => content_type,
    }
}
/// Public encoder for producing DBIN-like streams.
pub struct Encoder {
    config: EncoderConfig,
}

impl Encoder {
    const MAX_FRAME: usize = u32::MAX as usize; // 4_294_967_295 bytes (~4 GiB)
    const MAX_CT_LEN: usize = u16::MAX as usize; // 65 535 bytes

    /// Create a V0 encoder. Content type must be exactly 3 ASCII bytes (e.g. `"ETH"`).
    pub fn new_v0(content_type: &str, content_version: [u8; 2]) -> Self {
        let bytes = content_type.as_bytes();
        assert_eq!(bytes.len(), 3, "content_type must be 3 bytes for V0");
        let ct = [bytes[0], bytes[1], bytes[2]];
        Self {
            config: EncoderConfig::V0 {
                content_type: ct,
                content_version,
            },
        }
    }

    /// Create a V1 encoder with an arbitrary content type string.
    pub fn new_v1(content_type: &str) -> Self {
        let ct_bytes = content_type.as_bytes();
        assert!(
            ct_bytes.len() <= Self::MAX_CT_LEN,
            "content_type must be <= {} bytes for V1",
            Self::MAX_CT_LEN
        );
        Self {
            config: EncoderConfig::V1 {
                content_type: content_type.to_string(),
            },
        }
    }

    /// Serialize each item with `serialize`, optionally wrap it in a
    /// [`BstreamBlock`], then write header + frames to function `w`.
    ///
    /// Frame kind guidance:
    /// - Use [`FrameKind::Bstream`] when encoding blockchain blocks (e.g., Ethereum/Solana).
    ///   Items should serialize to the protobuf bytes expected by `BstreamBlock.payload`.
    /// - Use [`FrameKind::Raw`] for arbitrary, non-block payloads written as-is —
    ///   useful for storing and reusing objects such as SSZ states, beacon slots,
    ///   snapshots, or any other types that are not blocks (including ones not yet implemented).
    ///
    /// Notes:
    /// - With `Bstream`, the serialized bytes are wrapped in a `BstreamBlock` envelope.
    /// - The inner `Any` currently uses an Ethereum block `type_url`; adjust as needed
    ///   if you emit a different block type.
    pub fn encode_with<I, T, W, S>(
        &self,
        mut w: W,
        items: I,
        frame_kind: FrameKind,
        mut serialize: S,
    ) -> io::Result<()>
    where
        I: IntoIterator<Item = T>,
        W: Write,
        S: FnMut(T) -> Vec<u8>,
    {
        self.write_header_io(&mut w)?;
        let header_ct = self.config.content_type_str()?;
        let ct_url = type_url_for(header_ct);
        if let FrameKind::Bstream = frame_kind {
            if ct_url != ETH_HEADER && ct_url != SOLANA_HEADER {
                warn!(
                    "FrameKind::Bstream with unrecognized content_type `{}`; expected ETH or Solana blocks. Proceeding.",
                    header_ct
                );
            }
        }
        for item in items {
            let mut bytes = serialize(item);
            if let FrameKind::Bstream = frame_kind {
                let any = prost_wkt_types::Any {
                    type_url: ct_url.to_string(),
                    value: bytes,
                };
                bytes = BstreamBlock {
                    payload: Some(any),
                    ..Default::default()
                }
                .encode_to_vec();
            }

            self.write_frame_io(&mut w, &bytes)?;
        }
        Ok(())
    }

    /// Encode Prost messages (e.g. [`firehose_protos::EthBlock`], `SolBlock`) as Bstream frames to any `Write`.
    pub fn encode_prost_blocks_to_writer<I, M, W>(&self, mut w: W, blocks: I) -> io::Result<()>
    where
        I: IntoIterator<Item = M>,
        M: Message,
        W: Write,
    {
        self.encode_with(&mut w, blocks, FrameKind::Bstream, |m| m.encode_to_vec())
    }

    /// Encode one SSZ value (e.g. BeaconState) as a single raw frame to any `Write`.
    pub fn encode_ssz_value_to_writer<W, T: ssz::Encode>(
        &self,
        mut w: W,
        value: &T,
    ) -> io::Result<()>
    where
        W: Write,
    {
        let mut buf = Vec::with_capacity(value.ssz_bytes_len());
        value.ssz_append(&mut buf);
        self.encode_with(&mut w, std::iter::once(buf), FrameKind::Raw, |b| b)
    }

    /// Encode already-prepared byte slices as raw frames to any `Write`.
    pub fn encode_bytes_to_writer<I, B, W>(&self, mut w: W, frames: I) -> io::Result<()>
    where
        I: IntoIterator<Item = B>,
        B: AsRef<[u8]>,
        W: Write,
    {
        self.write_header_io(&mut w)?;
        for b in frames {
            self.write_frame_io(&mut w, b.as_ref())?;
        }
        Ok(())
    }

    fn write_header_io<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_all(b"dbin")?;
        match &self.config {
            EncoderConfig::V0 {
                content_type,
                content_version,
            } => {
                w.write_all(&[0u8])?;
                w.write_all(content_type)?;
                w.write_all(content_version)?;
            }
            EncoderConfig::V1 { content_type } => {
                w.write_all(&[1u8])?;
                let ct = content_type.as_bytes();
                w.write_all(&(ct.len() as u16).to_be_bytes())?;
                w.write_all(ct)?;
            }
        }
        Ok(())
    }

    fn write_frame_io<W: Write>(&self, w: &mut W, block: &[u8]) -> io::Result<()> {
        if block.len() > Self::MAX_FRAME {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "frame too large for u32",
            ));
        }
        let len_be = (block.len() as u32).to_be_bytes();
        w.write_all(&len_be)?;
        w.write_all(block)
    }
}
