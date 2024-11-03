use std::{ffi::OsStr, io::Read};

use crate::error::DecoderError;

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

/// `DbinHeader` contains the fields that compose the header of the .dbin file.
#[derive(Debug)]
pub struct DbinHeader {
    /// Next single byte after the 4 magic bytes, file format version
    pub version: u8,
    /// Next 3 bytes, content type like 'ETH', 'EOS', or something else
    // WHY USE String here?
    pub content_type: String,
    /// Next 2 bytes, 10-based string representation of content version, ranges in '00'-'99'
    pub content_version: String,
}

impl DbinFile {
    /// Reads a DbinHeader, which is used as the starting point for interpreting .dbin file contents.
    fn read_header<R: Read>(read: &mut R) -> Result<DbinHeader, DecoderError> {
        // Read the 4 magic bytes
        let mut buf: [u8; 4] = [0; 4];
        read.read_exact(&mut buf)?;

        if !DbinFile::magic_bytes_valid(&buf) {
            return Err(DecoderError::DbinMagicBytesInvalid);
        }

        let dbin_header = Self::read_partial_header(read)?;

        Ok(dbin_header)
    }

    fn magic_bytes_valid(bytes: &DbinMagicBytes) -> bool {
        bytes == DBIN_MAGIC_BYTES
    }

    /// Reads all the fields that make a DbinHeader
    fn read_partial_header<R: Read>(read: &mut R) -> Result<DbinHeader, DecoderError> {
        let version;
        let content_type;
        let content_version;

        let mut buf: [u8; 1] = [0; 1];
        read.read_exact(&mut buf)?;

        if buf[0] == 0 {
            version = 0u8;
            let mut content_type_bytes: [u8; 3] = [0; 3];
            read.read_exact(&mut content_type_bytes)?;

            content_type =
                String::from_utf8(Vec::from(content_type_bytes)).map_err(DecoderError::Utf8)?;

            let mut content_version_bytes: [u8; 2] = [0; 2];
            read.read_exact(&mut content_version_bytes)?;

            content_version =
                String::from_utf8(Vec::from(content_version_bytes)).map_err(DecoderError::Utf8)?;
        } else {
            return Err(DecoderError::DbinVersionUnsupported);
        }

        Ok(DbinHeader {
            version,
            content_type,
            content_version,
        })
    }

    /// Returns a `DbinFile` from a Reader
    pub fn try_from_read<R: Read>(read: &mut R) -> Result<Self, DecoderError> {
        let dbin_header = Self::read_header(read)?;
        let mut messages: Vec<Vec<u8>> = vec![];

        loop {
            match Self::read_message(read) {
                Ok(message) => messages.push(message),
                Err(e) => {
                    match e {
                        DecoderError::Io(io_error) => {
                            if io_error.kind() == std::io::ErrorKind::UnexpectedEof {
                                return Ok(DbinFile {
                                    header: DbinHeader {
                                        version: dbin_header.version,
                                        content_type: dbin_header.content_type,
                                        content_version: dbin_header.content_version,
                                    },
                                    messages,
                                });
                            } else if io_error.kind() == std::io::ErrorKind::Other {
                                // Check that version, content_type, and content_version match the previous header
                                let dbin_header_new = Self::read_partial_header(read)?;
                                if dbin_header.version != dbin_header_new.version
                                    || dbin_header.content_type != dbin_header_new.content_type
                                    || dbin_header.content_version
                                        != dbin_header_new.content_version
                                {
                                    return Err(DecoderError::DifferingDbinVersions);
                                }
                            }
                        }
                        // Catch all other variants of the error
                        e => return Err(e),
                    }
                }
            }
        }
    }
}

impl DbinFile {
    /// Reads a single message
    fn read_message<R: Read>(read: &mut R) -> Result<Vec<u8>, DecoderError> {
        let mut size: [u8; 4] = [0; 4];
        read.read_exact(&mut size)?;

        if &size == b"dbin" {
            return Err(DecoderError::DbinMagicBytesInvalid);
        }

        Ok(Self::read_content(size, read)?)
    }

    /// Reads a stream of messages.
    ///
    /// Messages are separated by "dbin" (magical 4 bytes) so each
    /// new occurrence of it marks the start of a new .dbin file
    pub fn read_message_from_stream<R: Read>(read: &mut R) -> Result<Vec<u8>, DecoderError> {
        let mut size: [u8; 4] = [0; 4];
        read.read_exact(&mut size)?;

        if &size == b"dbin" {
            _ = Self::read_partial_header(read)?;
            size = [0; 4];
            read.read_exact(&mut size)?;
        }

        Ok(Self::read_content(size, read)?)
    }

    /// reads message bytes
    fn read_content<R: Read>(size: [u8; 4], read: &mut R) -> Result<Vec<u8>, std::io::Error> {
        let size = u32::from_be_bytes(size);
        let mut content: Vec<u8> = vec![0; size as usize];
        read.read_exact(&mut content)?;
        Ok(content)
    }
}

pub fn file_extension_is_dbin(extension: Option<&OsStr>) -> bool {
    const DBIN_EXTENSION: &str = "dbin";

    match extension {
        Some(ext) => ext == DBIN_EXTENSION,
        None => false,
    }
}
