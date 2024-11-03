use std::io::{self, Read};

use crate::error::DecoderError;

/// Dbin file type extension
pub const DBIN_EXTENSION: &str = "dbin";

/// The bytes of a dbin file minus the header
pub type DbinMessages = Vec<Vec<u8>>;

/// Each dbin message message is length-prefixed as 4 bytes big-endian uint32
const DBIN_MAGIC_BYTES: &[u8; 4] = b"dbin";

type DbinMagicBytes = [u8; 4];

/// `DbinFile` is a struct representing a simple file storage format to pack a stream of protobuf messages, defined by StreamingFast.
///
/// For more information, see [the dbin format documentation](https://github.com/streamingfast/dbin?tab=readme-ov-file).
#[derive(Debug)]
pub struct DbinFile {
    pub header: DbinHeader,
    pub messages: DbinMessages,
}

impl DbinFile {
    /// Reads and parses a `.dbin` file from any `Read` type, such as a file or a network stream.
    pub fn try_from_read<R: Read>(mut read: R) -> Result<Self, DecoderError> {
        let header = DbinHeader::try_from_read(&mut read)?;
        let messages = Self::read_messages(&mut read)?;
        Ok(Self { header, messages })
    }

    /// Reads all messages in the `.dbin` file after the header.
    fn read_messages<R: Read>(read: &mut R) -> Result<DbinMessages, DecoderError> {
        let mut messages = Vec::new();
        loop {
            match Self::read_message(read) {
                Ok(message) => messages.push(message),
                Err(DecoderError::Io(ref e)) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
        }
        Ok(messages)
    }

    /// Reads a single message, assuming the size prefix format defined by `.dbin`.
    fn read_message<R: Read>(read: &mut R) -> Result<Vec<u8>, DecoderError> {
        const DBIN_PREFIX_SIZE: usize = 4;

        let mut size = [0; DBIN_PREFIX_SIZE];
        read.read_exact(&mut size)?;
        let size = u32::from_be_bytes(size);

        let mut content = vec![0; size as usize];
        read.read_exact(&mut content)?;
        Ok(content)
    }
}

/// Header of a `.dbin` file, containing metadata such as version, content type, and content version.
#[derive(Debug)]
pub struct DbinHeader {
    /// Next single byte after the 4 magic bytes, file format version
    pub version: u8,
    /// Next 3 bytes, content type like 'ETH', 'EOS', or something else
    pub content_type: String,
    /// Next 2 bytes, 10-based string representation of content version, ranges in '00'-'99'
    pub content_version: String,
}

impl DbinHeader {
    /// Reads the `.dbin` header from the given `Read` source, validating magic bytes and reading metadata.
    fn try_from_read<R: Read>(read: &mut R) -> Result<Self, DecoderError> {
        // Read the 4 magic bytes
        let mut buf: [u8; 4] = [0; 4];
        read.read_exact(&mut buf)?;

        if !Self::magic_bytes_valid(&buf) {
            return Err(DecoderError::DbinMagicBytesInvalid);
        }

        let dbin_header = Self::try_from_read_inner(read)?;

        Ok(dbin_header)
    }

    /// Reads from a stream of messages, returning the next message.
    ///
    /// Messages are separated by "dbin" (magical 4 bytes) so each
    /// new occurrence of it marks the start of a new `.dbin` file
    pub fn read_message_from_stream<R: Read>(read: &mut R) -> Result<Vec<u8>, DecoderError> {
        let mut bytes = Self::read_magic_bytes(read)?;

        if Self::magic_bytes_valid(&bytes) {
            _ = Self::try_from_read_inner(read)?;
            bytes = Self::read_magic_bytes(read)?;
        }

        Ok(Self::read_content(bytes, read)?)
    }

    /// Reads all the fields that make a DbinHeader
    fn try_from_read_inner<R: Read>(read: &mut R) -> Result<Self, DecoderError> {
        let mut buf: [u8; 1] = [0; 1];
        read.read_exact(&mut buf)?;

        if buf[0] == 0 {
            let version = 0u8;
            let mut content_type_bytes: [u8; 3] = [0; 3];
            read.read_exact(&mut content_type_bytes)?;

            let content_type =
                String::from_utf8(Vec::from(content_type_bytes)).map_err(DecoderError::Utf8)?;

            let mut content_version_bytes: [u8; 2] = [0; 2];
            read.read_exact(&mut content_version_bytes)?;

            let content_version =
                String::from_utf8(Vec::from(content_version_bytes)).map_err(DecoderError::Utf8)?;

            Ok(DbinHeader {
                version,
                content_type,
                content_version,
            })
        } else {
            return Err(DecoderError::DbinVersionUnsupported);
        }
    }

    fn magic_bytes_valid(bytes: &DbinMagicBytes) -> bool {
        bytes == DBIN_MAGIC_BYTES
    }

    /// Reads message bytes
    fn read_content<R: Read>(size: [u8; 4], read: &mut R) -> Result<Vec<u8>, std::io::Error> {
        let size = u32::from_be_bytes(size);
        let mut content: Vec<u8> = vec![0; size as usize];
        read.read_exact(&mut content)?;
        Ok(content)
    }

    fn read_magic_bytes<R: Read>(read: &mut R) -> Result<DbinMagicBytes, DecoderError> {
        let mut buf = [0; 4];
        read.read_exact(&mut buf)?;
        Ok(buf)
    }
}
