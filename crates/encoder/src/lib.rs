// Encoder crate — DBIN-aligned encoder.
//
// This crate encodes raw block data into a DBIN-like binary stream that can be
// consumed by the decoder in `crates/decoder`.

use std::fs::File;
use std::io::{self, Write};

use firehose_protos::BstreamBlock;
use prost::Message;

/// Encoder version selector for the DBIN header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Version {
    V0 = 0,
    V1 = 1,
}

/// Frame encoding mode.
#[derive(Debug, Clone, Copy)]
pub enum FrameKind {
    /// Frame bytes are written directly.
    Raw,
    /// Frame bytes are wrapped in a `BstreamBlock` protobuf.
    Bstream,
}

fn type_url_for(content_type: &str) -> &str {
    match content_type {
        // EVM
        "ETH" | "type.googleapis.com/sf.ethereum.type.v2.Block" => {
            "type.googleapis.com/sf.ethereum.type.v2.Block"
        }
        // Solana
        "type.googleapis.com/sf.solana.type.v1.Block" => {
            "type.googleapis.com/sf.solana.type.v1.Block"
        }
        // default: use the header value verbatim
        other => other,
    }
}

/// Public encoder for producing DBIN-like streams.
pub struct Encoder {
    version: Version,
    content_type: String,
    // Only used for V0
    content_version: [u8; 2],
}

impl Encoder {
    const MAX_FRAME: usize = u32::MAX as usize; // 4_294_967_295 bytes (~4 GiB)
    const MAX_CT_LEN: usize = u16::MAX as usize; // 65 535 bytes

    /// Create a V0 encoder. Content type must be exactly 3 ASCII bytes (e.g. `"ETH"`).
    pub fn new_v0(content_type: &str, content_version: [u8; 2]) -> Self {
        assert_eq!(content_type.len(), 3, "content_type must be 3 bytes for V0");
        Self {
            version: Version::V0,
            content_type: content_type.to_string(),
            content_version,
        }
    }

    /// Create a V1 encoder with an arbitrary content type string.
    pub fn new_v1(content_type: &str) -> Self {
        Self {
            version: Version::V1,
            content_type: content_type.to_string(),
            content_version: [0u8; 2],
        }
    }

    // ------------------------------------------------------------------------
    // Core generic encoder
    // ------------------------------------------------------------------------

    /// Serialize each item with `serialize`, optionally wrap it in a
    /// `BstreamBlock`, then write header + frames to `w`.
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
        for item in items {
            let mut bytes = serialize(item);
            if let FrameKind::Bstream = frame_kind {
                // wrap inner bytes in an Any with the EthBlock type URL
                let any = prost_wkt_types::Any {
                    type_url: "type.googleapis.com/firehose_protos.ethereum_v2.eth_block.EthBlock"
                        .to_string(),
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

    // ------------------------------------------------------------------------
    // Convenience helpers
    // ------------------------------------------------------------------------

    /// Encode Prost messages (e.g. `EthBlock`, `SolBlock`) as Bstream frames to a file.
    pub fn encode_prost_blocks_to_path<I, M>(&self, path: &str, blocks: I) -> io::Result<()>
    where
        I: IntoIterator<Item = M>,
        M: Message,
    {
        let mut f = File::create(path)?;
        self.encode_with(&mut f, blocks, FrameKind::Bstream, |m| m.encode_to_vec())
    }

    /// Encode one SSZ value (e.g. BeaconState) as a single raw frame to a file.
    pub fn encode_ssz_value_to_path<T: ssz::Encode>(
        &self,
        path: &str,
        value: &T,
    ) -> io::Result<()> {
        let mut f = File::create(path)?;
        let mut buf = Vec::with_capacity(value.ssz_bytes_len());
        value.ssz_append(&mut buf);
        self.encode_with(&mut f, std::iter::once(buf), FrameKind::Raw, |b| b)
    }

    /// Encode already-prepared byte slices as raw frames to a file.
    pub fn encode_bytes_to_path<I, B>(&self, path: &str, frames: I) -> io::Result<()>
    where
        I: IntoIterator<Item = B>,
        B: AsRef<[u8]>,
    {
        let mut f = File::create(path)?;
        self.write_header_io(&mut f)?;
        for b in frames {
            self.write_frame_io(&mut f, b.as_ref())?;
        }
        Ok(())
    }

    // ------------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------------

    fn write_header_io<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_all(b"dbin")?;
        w.write_all(&[self.version as u8])?;
        match self.version {
            Version::V0 => {
                debug_assert_eq!(self.content_type.len(), 3);
                w.write_all(self.content_type.as_bytes())?;
                w.write_all(&self.content_version)?;
            }
            Version::V1 => {
                let ct = self.content_type.as_bytes();
                if ct.len() > Self::MAX_CT_LEN {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "content_type too long",
                    ));
                }
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

// Identity passthrough for legacy code.
pub fn encode(input: &[u8]) -> Vec<u8> {
    input.to_vec()
}
