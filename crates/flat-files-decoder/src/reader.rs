use std::{ffi::OsStr, io::Read};

use crate::{
    dbin::{DbinFile, DBIN_EXTENSION},
    error::DecoderError,
};

/// `BlockFileReader` is an enum that supports reading files in different formats.
/// Initially, it supports only the `Dbin` format. Additional formats can be added as new variants.
#[derive(Debug)]
pub enum BlockFileReader {
    Dbin(DbinFile),
}

impl BlockFileReader {
    /// Attempts to read a block file based on the provided reader and file extension.
    pub fn try_from_read<R: Read>(
        extension: Option<&OsStr>,
        read: &mut R,
    ) -> Result<Self, DecoderError> {
        match extension {
            Some(ext) if Self::file_extension_is_dbin(Some(ext)) => {
                let dbin_file = DbinFile::try_from_read(read)?;
                Ok(BlockFileReader::Dbin(dbin_file))
            }
            Some(ext) => Err(DecoderError::FormatUnsupported(Some(
                ext.to_string_lossy().into_owned(),
            ))),
            None => Err(DecoderError::FormatUnsupported(None)),
        }
    }

    /// Check whether an optional reference to an [`OsStr`] represents a `.dbin` file extension.
    pub fn file_extension_is_dbin(extension: Option<&OsStr>) -> bool {
        match extension {
            Some(ext) => ext == DBIN_EXTENSION,
            None => false,
        }
    }
}
