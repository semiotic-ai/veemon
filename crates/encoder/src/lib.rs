//! Encoder crate — DBIN-aligned encoder.
//!
//! This crate encodes raw block data into a DBIN-like binary stream that can be consumed by the decoder in `crates/decoder`.

/// Encoder version selector for the DBIN header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Version {
    V0 = 0,
    V1 = 1,
}

/// Public encoder for producing DBIN-like streams.
pub struct Encoder {
    version: Version,
    content_type: String,
    // Only used for V0
    content_version: [u8; 2],
}

impl Encoder {
    /// Create a V0 encoder. Content type must be exactly 3 ASCII bytes (e.g., "ETH").
    pub fn new_v0(content_type: &str, content_version: [u8; 2]) -> Self {
        assert_eq!(content_type.as_bytes().len(), 3, "content_type must be 3 bytes for V0");
        Self {
            version: Version::V0,
            content_type: content_type.to_string(),
            content_version,
        }
    }

    /// Create a V1 encoder with a given content type.
    pub fn new_v1(content_type: &str) -> Self {
        Self {
            version: Version::V1,
            content_type: content_type.to_string(),
            content_version: [0u8; 2],
        }
    }

    /// Encode a single block into a DBIN-style stream: header followed by a single framed block.
    pub fn encode_block(&self, block: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        self.write_header(&mut out);
        self.write_frame(&mut out, block);
        out
    }

    /// Encode a sequence of blocks into a single stream (header + frames).
    pub fn encode_blocks<I>(&self, blocks: I) -> Vec<u8>
    where
        I: IntoIterator<Item = Vec<u8>>,
    {
        let mut out = Vec::new();
        self.write_header(&mut out);
        for b in blocks {
            self.write_frame(&mut out, &b);
        }
        out
    }

    /// Convenience wrapper to encode a stream of blocks with header.
    pub fn wrap_stream<I>(&self, blocks: I) -> Vec<u8>
    where
        I: IntoIterator<Item = Vec<u8>>,
    {
        self.encode_blocks(blocks)
    }

    // internal helpers
    fn write_header(&self, out: &mut Vec<u8>) {
        // magic
        out.extend_from_slice(b"dbin");
        // version
        out.push(self.version as u8);
        match self.version {
            Version::V0 => {
                // 3-byte content type
                out.extend_from_slice(self.content_type.as_bytes());
                // 2-byte content version
                out.extend_from_slice(&self.content_version);
            }
            Version::V1 => {
                // 2-byte content type length + content type
                let ct = self.content_type.as_bytes();
                let len = (ct.len() as u16).to_be_bytes();
                out.extend_from_slice(&len);
                out.extend_from_slice(ct);
            }
        }
    }

    fn write_frame(&self, out: &mut Vec<u8>, block: &[u8]) {
        let len = (block.len() as u32).to_be_bytes();
        out.extend_from_slice(&len);
        out.extend_from_slice(block);
    }
}

/// Identity encode for compatibility with existing usage.
pub fn encode(input: &[u8]) -> Vec<u8> {
    input.to_vec()
}

// NEW: expose a generic encoding helper for DBIN from blocks (ETH blocks by default)
pub mod encode_utils;

