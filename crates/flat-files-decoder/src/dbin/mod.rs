pub mod error;

use crate::dbin::error::DbinFileError;
use std::io::Read;

pub struct DbinFile {
    pub version: u8,
    pub content_type: String,
    pub content_version: String,
    pub messages: Vec<Vec<u8>>,
}

pub struct DbinHeader {
    pub version: u8,
    pub content_type: String,
    pub content_version: String,
}

impl DbinFile {
    fn read_header<R: Read>(read: &mut R) -> Result<DbinHeader, DbinFileError> {
        let mut buf: [u8; 4] = [0; 4];
        read.read_exact(&mut buf)
            .map_err(DbinFileError::ReadError)?;

        let dbin_header = Self::read_partial_header(read)?;

        Ok(dbin_header)
    }

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

    pub fn try_from_read<R: Read>(read: &mut R) -> Result<Self, DbinFileError> {
        let dbin_header = Self::read_header(read)?;
        let mut messages: Vec<Vec<u8>> = vec![];

        loop {
            match Self::read_message(read) {
                Ok(message) => messages.push(message),
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::UnexpectedEof {
                        return Ok(DbinFile {
                            version: dbin_header.version,
                            content_type: dbin_header.content_type,
                            content_version: dbin_header.content_version,
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
    pub fn read_message<R: Read>(read: &mut R) -> Result<Vec<u8>, DbinFileError> {
        let mut size: [u8; 4] = [0; 4];
        read.read_exact(&mut size)?;

        if &size == b"dbin" {
            return Err(DbinFileError::StartOfNewDBINFile);
        }

        Ok(Self::read_content(size, read)?)
    }

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

    fn read_content<R: Read>(size: [u8; 4], read: &mut R) -> Result<Vec<u8>, std::io::Error> {
        let size = u32::from_be_bytes(size);
        let mut content: Vec<u8> = vec![0; size as usize];
        read.read_exact(&mut content)?;
        Ok(content)
    }
}
