// Encoder crate — DBIN-aligned encoder.
//
// This crate encodes raw block data into a DBIN-like binary stream that can be consumed by the decoder in `crates/decoder`.

use std::fs::File;
use std::io::{self, Cursor, Write};

use firehose_protos::{BstreamBlock, EthBlock, SolBlock};
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
    const MAX_FRAME: usize = u32::MAX as usize; // 4_294_967_295
    const MAX_CT_LEN: usize = u16::MAX as usize;

    /// Create a V0 encoder. Content type must be exactly 3 ASCII bytes (e.g., "ETH").
    pub fn new_v0_encoder(content_type: &str, content_version: [u8; 2]) -> Self {
        assert_eq!(content_type.len(), 3, "content_type must be 3 bytes for V0");
        Self {
            version: Version::V0,
            content_type: content_type.to_string(),
            content_version,
        }
    }

    /// Create a V1 encoder with a given content type.
    pub fn new_v1_encoder(content_type: &str) -> Self {
        Self {
            version: Version::V1,
            content_type: content_type.to_string(),
            content_version: [0u8; 2],
        }
    }

    // Backward-compat alias for tests/usages that expect `new_v1`.
    pub fn new_v1(content_type: &str) -> Self {
        Self::new_v1_encoder(content_type)
    }

    // ---------------- Streaming APIs only ----------------

    /// Write header + frames (bytes) to a writer.
    pub fn encode_bytes_to_writer<I, B, W>(&self, mut w: W, blocks: I) -> io::Result<()>
    where
        I: IntoIterator<Item = B>,
        B: AsRef<[u8]>,
        W: Write,
    {
        self.write_header_io(&mut w)?;
        for b in blocks {
            self.write_frame_io(&mut w, b.as_ref())?;
        }
        Ok(())
    }

    /// Convenience: write bytes frames to a filesystem path.
    pub fn encode_bytes_to_path<I, B>(&self, path: &str, blocks: I) -> io::Result<()>
    where
        I: IntoIterator<Item = B>,
        B: AsRef<[u8]>,
    {
        let mut f = File::create(path)?;
        self.encode_bytes_to_writer(&mut f, blocks)
    }

    /// Write header + frames (Prost messages) to a writer.
    pub fn encode_blocks_to_writer<I, M, W>(&self, mut w: W, blocks: I) -> io::Result<()>
    where
        I: IntoIterator<Item = M>,
        M: prost::Message,
        W: Write,
    {
        self.write_header_io(&mut w)?;
        for b in blocks {
            let bytes = b.encode_to_vec();
            self.write_frame_io(&mut w, &bytes)?;
        }
        Ok(())
    }

    /// Convenience: write Prost messages to a filesystem path.
    pub fn encode_blocks_to_path<I, M>(&self, path: &str, blocks: I) -> io::Result<()>
    where
        I: IntoIterator<Item = M>,
        M: prost::Message,
    {
        let mut f = File::create(path)?;
        self.encode_blocks_to_writer(&mut f, blocks)
    }

    /// Write one SSZ-encodable value as a single frame to a writer.
    pub fn encode_value_to_writer<T: ssz::Encode, W: Write>(
        &self,
        mut w: W,
        value: &T,
    ) -> io::Result<()> {
        self.write_header_io(&mut w)?;
        let mut frame = Vec::with_capacity(value.ssz_bytes_len());
        value.ssz_append(&mut frame);
        self.write_frame_io(&mut w, &frame)
    }

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

    // Backward-compatible wrappers for older API names used in tests/examples
    pub fn encode_block<M: prost::Message>(&self, block: M) -> Vec<u8> {
        let mut out = Vec::new();
        let mut w = Cursor::new(&mut out);
        self.encode_blocks_to_writer(&mut w, std::iter::once(block))
            .unwrap();
        out
    }
}
