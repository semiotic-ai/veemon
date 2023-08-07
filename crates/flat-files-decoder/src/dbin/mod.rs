pub mod error;

use crate::dbin::error::DbinFileError;
use std::io::Read;

pub struct DbinFile {
    pub version: u8,
    pub content_type: String,
    pub content_version: String,
    pub messages: Vec<Vec<u8>>,
}

impl DbinFile {
    pub fn try_from_read<R: Read>(read: &mut R) -> Result<Self, DbinFileError> {
        let mut dbin: [u8; 4] = [0; 4];
        read.read_exact(&mut dbin)
            .map_err(DbinFileError::ReadError)?;

        let dbin = String::from_utf8(Vec::from(dbin)).map_err(DbinFileError::InvalidUTF8)?;

        if dbin != "dbin" {
            return Err(DbinFileError::InvalidDBINBytes);
        }

        let mut version: [u8; 1] = [0];
        read.read_exact(&mut version)
            .map_err(DbinFileError::ReadError)?;

        let mut content_type: [u8; 3] = [0; 3];
        read.read_exact(&mut content_type)
            .map_err(DbinFileError::ReadError)?;

        let content_type =
            String::from_utf8(Vec::from(content_type)).map_err(DbinFileError::InvalidUTF8)?;

        let mut content_version: [u8; 2] = [0; 2];
        read.read_exact(&mut content_version)
            .map_err(DbinFileError::ReadError)?;

        let content_version =
            String::from_utf8(Vec::from(content_version)).map_err(DbinFileError::InvalidUTF8)?;

        let mut messages: Vec<Vec<u8>> = vec![];

        loop {
            match Self::read_message(read.by_ref()) {
                Ok(message) => messages.push(message),
                Err(err) => {
                    return if err.kind() == std::io::ErrorKind::UnexpectedEof {
                        Ok(DbinFile {
                            version: version[0],
                            content_type,
                            content_version,
                            messages,
                        })
                    } else {
                        Err(DbinFileError::ReadError(err))
                    }
                }
            }
        }
    }
}

impl DbinFile {
    fn read_message<R: Read>(read: &mut R) -> Result<Vec<u8>, std::io::Error> {
        let mut size: [u8; 4] = [0; 4];
        read.read_exact(&mut size)?;

        let size = u32::from_be_bytes(size);

        let mut content: Vec<u8> = vec![0; size as usize];
        read.read_exact(&mut content)?;

        Ok(content)
    }
}
