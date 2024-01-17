pub mod error;

use crate::dbin::error::DbinFileError;
use std::io::Read;

/// `DbinFile` is a struct that represents a simple file storage format to pack a stream of protobuf messages. It is defined by StreamingFast.
///
/// For more information, see [the dbin format documentation](https://github.com/streamingfast/dbin?tab=readme-ov-file).
pub struct DbinFile {
    pub header: DbinHeader,
    /// Rest of the bytes of the file, each message is length-prefixed as 4 bytes big-endian uin32
    pub messages: Vec<Vec<u8>>,
}

/// `DbinHeader` contains the fields that compose the header of the .dbin file.
pub struct DbinHeader {
    /// Next single byte after the 4 magic bytes, file format version
    pub version: u8,
    /// Next 3 bytes, content type like 'ETH', 'EOS', or something else
    pub content_type: String,
    /// Next 2 bytes, 10-based string representation of content version, ranges in '00'-'99'
    pub content_version: String,
}

//TODO: why not nest DbinHeader inside DbinFile?

impl DbinFile {
    /// reads a DbinHeader
    ///
    /// It nests `read_partial_header` to read header. By itself, it reads the 4 magic bytes
    fn read_header<R: Read>(read: &mut R) -> Result<DbinHeader, DbinFileError> {
        let mut buf: [u8; 4] = [0; 4];

        read.read_exact(&mut buf)
            .map_err(DbinFileError::ReadError)?;

        if &buf != b"dbin" {
            return Err(DbinFileError::StartOfNewDBINFile);
        }

        let dbin_header = Self::read_partial_header(read)?;

        Ok(dbin_header)
    }

    /// Reads all the fields that make a DbinHeader
    fn read_partial_header<R: Read>(read: &mut R) -> Result<DbinHeader, DbinFileError> {
        let version;
        let content_type;
        let content_version;

        let mut buf: [u8; 1] = [0; 1];
        read.read_exact(&mut buf)
            .map_err(DbinFileError::ReadError)?;

        if buf[0] == 0 {
            version = 0u8;
            let mut content_type_bytes: [u8; 3] = [0; 3];
            read.read_exact(&mut content_type_bytes)
                .map_err(DbinFileError::ReadError)?;

            content_type = String::from_utf8(Vec::from(content_type_bytes))
                .map_err(DbinFileError::InvalidUTF8)?;

            let mut content_version_bytes: [u8; 2] = [0; 2];
            read.read_exact(&mut content_version_bytes)
                .map_err(DbinFileError::ReadError)?;

            content_version = String::from_utf8(Vec::from(content_version_bytes))
                .map_err(DbinFileError::InvalidUTF8)?;
        } else {
            return Err(DbinFileError::UnsupportedDBINVersion);
        }

        Ok(DbinHeader {
            version,
            content_type,
            content_version,
        })
    }

    /// Returns a `DbinFile` from a Reader
    pub fn try_from_read<R: Read>(read: &mut R) -> Result<Self, DbinFileError> {
        let dbin_header = Self::read_header(read)?;
        let mut messages: Vec<Vec<u8>> = vec![];

        loop {
            match Self::read_message(read) {
                Ok(message) => messages.push(message),
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::UnexpectedEof {
                        return Ok(DbinFile {
                            header: DbinHeader {
                                version: dbin_header.version,
                                content_type: dbin_header.content_type,
                                content_version: dbin_header.content_version,
                            },
                            messages,
                        });
                    } else if err.kind() == std::io::ErrorKind::Other {
                        // Check that version, content_type, and content_version match the previous header
                        let dbin_header_new = Self::read_partial_header(read)?;
                        if dbin_header.version != dbin_header_new.version
                            || dbin_header.content_type != dbin_header_new.content_type
                            || dbin_header.content_version != dbin_header_new.content_version
                        {
                            return Err(DbinFileError::DifferingDBINVersions);
                        }
                    } else {
                        return Err(err);
                    }
                }
            }
        }
    }
}

impl DbinFile {
    /// Reads a single message
    pub fn read_message<R: Read>(read: &mut R) -> Result<Vec<u8>, DbinFileError> {
        let mut size: [u8; 4] = [0; 4];
        read.read_exact(&mut size)?;

        if &size == b"dbin" {
            return Err(DbinFileError::StartOfNewDBINFile);
        }

        Ok(Self::read_content(size, read)?)
    }

    /// Reads a stream of messages.
    ///
    /// Messages are separated by "dbin" (magical 4 bytes) so each
    /// new occurrence of it marks the start of a new .dbin file
    pub fn read_message_stream<R: Read>(read: &mut R) -> Result<Vec<u8>, DbinFileError> {
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
