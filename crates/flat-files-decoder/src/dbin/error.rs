use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbinFileError {
    #[error("Incorrect dbin bytes")]
    InvalidDBINBytes,
    #[error("Read error")]
    ReadError(#[from] std::io::Error),
    #[error("Invalid UTF8")]
    InvalidUTF8(#[from] std::string::FromUtf8Error),
    #[error("Unsupported version")]
    UnsupportedDBINVersion,
    #[error("Start of new DBIN file")]
    StartOfNewDBINFile,
    #[error("DBIN files with different versions")]
    DifferingDBINVersions,
}

impl DbinFileError {
    pub fn kind(&self) -> std::io::ErrorKind {
        match self {
            DbinFileError::StartOfNewDBINFile => std::io::ErrorKind::Other,
            DbinFileError::InvalidDBINBytes => todo!(),
            DbinFileError::ReadError(_) => std::io::ErrorKind::UnexpectedEof,
            DbinFileError::InvalidUTF8(_) => todo!(),
            DbinFileError::UnsupportedDBINVersion => todo!(),
            DbinFileError::DifferingDBINVersions => todo!(),
        }
    }
}
