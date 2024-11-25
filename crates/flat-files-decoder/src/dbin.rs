// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::io::{self, Read};

use crate::error::DecoderError;

/// The bytes of a dbin file minus the header
type DbinMessages = Vec<DbinMessage>;

/// The bytes of a dbin message
type DbinMessage = Vec<u8>;

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

/// Work with a `.dbin` flat file.
///
/// Developed by StreamingFast, dbin is a simple file storage format to pack a stream of protobuffer messages.
/// For more information, see [the dbin format documentation](https://github.com/streamingfast/dbin?tab=readme-ov-file).
#[derive(Debug)]
pub struct DbinFile {
    header: DbinHeader,
    messages: DbinMessages,
}

impl DbinFile {
    /// Get the content type of the `.dbin` file, such as `"ETH"`.
    pub fn content_type(&self) -> &str {
        &self.header.content_type
    }

    /// Read and parse a `.dbin` file from a `Read` source.
    pub fn try_from_read<R: Read>(mut read: R) -> Result<Self, DecoderError> {
        let header = DbinHeader::try_from_read(&mut read)?;
        if !header.is_supported_version() {
            return Err(DecoderError::VersionUnsupported);
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

            match read_message(read, message_length) {
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
}

// implement iterator for DbinFile so that we can iterate over the messages
impl IntoIterator for DbinFile {
    type Item = Vec<u8>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.into_iter()
    }
}

/// Header of a `.dbin` file, containing metadata such as version, content type, and content version.
#[derive(Debug)]
struct DbinHeader {
    /// File format version, the next single byte after the 4 [`DbinMagicBytes`]
    version: u8,
    /// Content type like 'ETH', 'EOS', or something else; the next 3 bytes
    content_type: String,
}

impl DbinHeader {
    /// Checks if the version is supported.
    fn is_supported_version(&self) -> bool {
        is_supported_version(self.version)
    }

    /// Reads and validates the `.dbin` header from the given [`Read`] source.
    fn try_from_read<R: Read>(read: &mut R) -> Result<Self, DecoderError> {
        let magic_bytes = read_magic_bytes(read)?;
        if !magic_bytes_valid(&magic_bytes) {
            return Err(DecoderError::MagicBytesInvalid);
        }
        read_header(read)
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

fn is_supported_version(version: u8) -> bool {
    version == SUPPORTED_DBIN_VERSION
}

fn magic_bytes_valid(bytes: &MagicBytes) -> bool {
    bytes == MAGIC_BYTES
}

/// Reads and constructs a [`DbinHeader`] from the remaining fields after the magic bytes.
fn read_header<R: Read>(read: &mut R) -> Result<DbinHeader, DecoderError> {
    let version = match DbinHeader::read_version_field(read) {
        Ok(version) if is_supported_version(version) => version,
        Ok(_) => return Err(DecoderError::VersionUnsupported),
        Err(e) => return Err(e),
    };

    let content_type = DbinHeader::read_string_field(read, HEADER_CONTENT_TYPE_SIZE)?;

    // Content version, represented as 10-based string, ranges in '00'-'99'; the next 2 bytes
    let _content_version = DbinHeader::read_string_field(read, HEADER_CONTENT_VERSION_SIZE)?;

    Ok(DbinHeader {
        version,
        content_type,
    })
}

fn read_magic_bytes<R: Read>(read: &mut R) -> Result<MagicBytes, DecoderError> {
    let bytes = read_message(read, PREFIX_SIZE)?;
    match bytes.try_into() {
        Ok(magic_bytes) => Ok(magic_bytes),
        Err(_) => Err(DecoderError::MagicBytesInvalid),
    }
}

/// Reads a single message, assuming the size-prefix format defined by `.dbin`.
fn read_message<R: Read>(read: &mut R, length: usize) -> Result<DbinMessage, DecoderError> {
    let mut message = vec![0; length];
    read.read_exact(&mut message)?;
    Ok(message)
}

/// Read the next block from a flat file reader.
pub fn read_block_from_reader<R: Read>(read: &mut R) -> Result<DbinMessage, DecoderError> {
    let mut magic_bytes = read_magic_bytes(read)?;

    if magic_bytes_valid(&magic_bytes) {
        // Block messages are separated by "dbin" (the magical 4 bytes), so each
        // new occurrence marks the start of a new .dbin file
        _ = read_header(read)?;
        magic_bytes = read_magic_bytes(read)?;
    }

    let message_size = u32::from_be_bytes(magic_bytes) as usize;

    read_message(read, message_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_valid_header_parsing() {
        let data = [b'd', b'b', b'i', b'n', 0u8, b'E', b'T', b'H', b'0', b'1'];
        let mut cursor = Cursor::new(data);

        let header = DbinHeader::try_from_read(&mut cursor).expect("Failed to parse header");
        assert_eq!(header.version, SUPPORTED_DBIN_VERSION);
        assert_eq!(header.content_type, "ETH");
    }

    #[test]
    fn test_unsupported_version() {
        let data = [b'd', b'b', b'i', b'n', 1u8, b'E', b'T', b'H', b'0', b'1'];
        let mut cursor = Cursor::new(data);

        let result = DbinHeader::try_from_read(&mut cursor);
        assert!(matches!(result, Err(DecoderError::VersionUnsupported)));
    }

    #[test]
    fn test_invalid_magic_bytes() {
        let data = [b'x', b'y', b'z', b'n', 0u8, b'E', b'T', b'H', b'0', b'1'];
        let mut cursor = Cursor::new(data);

        let result = DbinHeader::try_from_read(&mut cursor);
        assert!(matches!(result, Err(DecoderError::MagicBytesInvalid)));
    }

    #[test]
    fn test_read_messages() {
        let mut data = vec![];
        data.extend_from_slice(&[b'd', b'b', b'i', b'n', 0u8, b'E', b'T', b'H', b'0', b'1']);
        data.extend_from_slice(&(4u32.to_be_bytes())); // message length
        data.extend_from_slice(b"test");

        let mut cursor = Cursor::new(data);
        let dbin_file = DbinFile::try_from_read(&mut cursor).expect("Failed to read dbin file");

        assert_eq!(dbin_file.messages.len(), 1);
        assert_eq!(dbin_file.messages[0], b"test");
    }

    #[test]
    fn test_end_of_file_handling() {
        let mut data = vec![];
        data.extend_from_slice(&[b'd', b'b', b'i', b'n', 0u8, b'E', b'T', b'H', b'0', b'1']);
        data.extend_from_slice(&(4u32.to_be_bytes())); // message length
        data.extend_from_slice(b"test");

        // truncate to simulate EOF after header
        let mut cursor = Cursor::new(&data[..data.len() - 2]);

        let result = DbinFile::try_from_read(&mut cursor);
        assert!(
            matches!(result, Err(DecoderError::Io(ref e)) if e.kind() == io::ErrorKind::UnexpectedEof)
        );
    }

    #[test]
    fn test_iterator_behavior() {
        let mut data = vec![];
        data.extend_from_slice(&[b'd', b'b', b'i', b'n', 0u8, b'E', b'T', b'H', b'0', b'1']);
        data.extend_from_slice(&(4u32.to_be_bytes())); // message length
        data.extend_from_slice(b"test");
        data.extend_from_slice(&(3u32.to_be_bytes())); // message length
        data.extend_from_slice(b"123");

        let mut cursor = Cursor::new(data);
        let dbin_file = DbinFile::try_from_read(&mut cursor).expect("Failed to read dbin file");

        let messages: Vec<_> = dbin_file.into_iter().collect();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], b"test");
        assert_eq!(messages[1], b"123");
    }
}
