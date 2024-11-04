use std::io::{self, Read};

use crate::error::DecoderError;

/// Dbin file type extension
pub const EXTENSION: &str = "dbin";

/// The bytes of a dbin file minus the header
pub type DbinMessages = Vec<Vec<u8>>;

/// Each dbin message is length-prefixed as 4 bytes big-endian uint32
const MAGIC_BYTES: &[u8; 4] = b"dbin";

/// The 4 magic bytes of a dbin file, indicating the file format
type MagicBytes = [u8; 4];

/// The size of the length prefix in bytes
const PREFIX_SIZE: usize = 4;

/// The size of the header version in bytes
const HEADER_VERSION_SIZE: usize = 1;

/// The size of the header content type in bytes
const HEADER_CONTENT_TYPE_SIZE: usize = 3;

/// The size of the header content version in bytes
const HEADER_CONTENT_VERSION_SIZE: usize = 2;

/// The supported version of the dbin file format
const SUPPORTED_DBIN_VERSION: u8 = 0;

/// `DbinFile` is a struct representing a simple file storage format to pack a stream of protobuf messages, defined by StreamingFast.
///
/// For more information, see [the dbin format documentation](https://github.com/streamingfast/dbin?tab=readme-ov-file).
#[derive(Debug)]
pub struct DbinFile {
    pub header: DbinHeader,
    pub messages: DbinMessages,
}

impl DbinFile {
    /// Reads and parses a `.dbin` file from a `Read` source.
    pub fn try_from_read<R: Read>(mut read: R) -> Result<Self, DecoderError> {
        let header = DbinHeader::try_from_read(&mut read)?;
        if !header.is_supported_version() {
            return Err(DecoderError::DbinVersionUnsupported);
        }
        let messages = Self::read_messages(&mut read)?;
        Ok(Self { header, messages })
    }

    /// Reads messages from a `Read` source following the Dbin format.
    fn read_messages<R: Read>(read: &mut R) -> Result<DbinMessages, DecoderError> {
        let mut messages = Vec::new();

        loop {
            let bytes = match read_magic_bytes(read) {
                Ok(bytes) => bytes,
                // Break loop gracefully if EOF is reached at the start of a new message.
                Err(DecoderError::Io(e)) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            let message_length = u32::from_be_bytes(bytes) as usize;

            match Self::read_message(read, message_length) {
                Ok(message) => messages.push(message),
                // Return error if EOF occurs in the middle of a message
                Err(DecoderError::Io(e)) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    return Err(DecoderError::Io(e))
                }
                Err(e) => return Err(e),
            }
        }

        Ok(messages)
    }

    /// Reads a single message, assuming the size-prefix format defined by `.dbin`.
    fn read_message<R: Read>(read: &mut R, length: usize) -> Result<Vec<u8>, DecoderError> {
        let mut message = vec![0; length];
        read.read_exact(&mut message)?;
        Ok(message)
    }
}

/// Header of a `.dbin` file, containing metadata such as version, content type, and content version.
#[derive(Debug)]
pub struct DbinHeader {
    /// File format version, the next single byte after the 4 [`DbinMagicBytes`]
    pub version: u8,
    /// Content type like 'ETH', 'EOS', or something else; the next 3 bytes
    pub content_type: String,
    /// Content version, represented as 10-based string, ranges in '00'-'99'; the next 2 bytes
    pub content_version: String,
}

impl DbinHeader {
    fn is_supported_version(&self) -> bool {
        self.version == SUPPORTED_DBIN_VERSION
    }

    /// Reads and validates the `.dbin` header from the given [`Read`] source.
    fn try_from_read<R: Read>(read: &mut R) -> Result<Self, DecoderError> {
        let magic_bytes = read_magic_bytes(read)?;
        if !Self::magic_bytes_valid(&magic_bytes) {
            return Err(DecoderError::DbinMagicBytesInvalid);
        }
        Self::try_from_read_inner(read)
    }

    /// Reads and constructs a [`DbinHeader`] from the remaining fields after the magic bytes.
    fn try_from_read_inner<R: Read>(read: &mut R) -> Result<Self, DecoderError> {
        let version = Self::read_version_field(read)?;
        let content_type = Self::read_string_field(read, HEADER_CONTENT_TYPE_SIZE)?;
        let content_version = Self::read_string_field(read, HEADER_CONTENT_VERSION_SIZE)?;

        Ok(DbinHeader {
            version,
            content_type,
            content_version,
        })
    }

    fn magic_bytes_valid(bytes: &MagicBytes) -> bool {
        bytes == MAGIC_BYTES
    }

    /// Reads message bytes
    fn read_message<R: Read>(read: &mut R, size: usize) -> Result<Vec<u8>, DecoderError> {
        let mut message = vec![0; size];
        read.read_exact(&mut message)?;
        Ok(message)
    }

    /// Reads from a stream of messages, returning the next message.
    ///
    /// Messages are separated by "dbin" (magical 4 bytes) so each
    /// new occurrence of it marks the start of a new `.dbin` file
    pub fn read_message_from_stream<R: Read>(read: &mut R) -> Result<Vec<u8>, DecoderError> {
        let mut magic_bytes = read_magic_bytes(read)?;

        if Self::magic_bytes_valid(&magic_bytes) {
            _ = Self::try_from_read_inner(read)?;
            magic_bytes = read_magic_bytes(read)?;
        }

        let message_size = u32::from_be_bytes(magic_bytes) as usize;

        Self::read_message(read, message_size)
    }

    fn read_string_field<R: Read>(read: &mut R, size: usize) -> Result<String, DecoderError> {
        let mut field_bytes = vec![0; size];
        read.read_exact(&mut field_bytes)?;
        String::from_utf8(field_bytes).map_err(DecoderError::from)
    }

    /// Reads a single byte as a version or field.
    fn read_version_field<R: Read>(read: &mut R) -> Result<u8, DecoderError> {
        let mut buf = [0; HEADER_VERSION_SIZE];
        read.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

fn read_magic_bytes<R: Read>(read: &mut R) -> Result<MagicBytes, DecoderError> {
    let mut buf = [0; PREFIX_SIZE];
    read.read_exact(&mut buf)?;
    Ok(buf)
}
