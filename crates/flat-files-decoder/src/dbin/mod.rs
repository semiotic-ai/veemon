use std::error::Error;
use std::fs::File;
use std::io::Read;

pub(crate) struct DbinFile {
    pub version: u8,
    pub content_type: String,
    pub content_version: String,
    pub messages: Vec<Vec<u8>>
}

impl DbinFile {
    pub(crate) fn from_file(mut file: File) -> Result<Self, Box<dyn Error>> {
        let mut dbin: [u8; 4] = [0; 4];
        file.read_exact(&mut dbin)?;

        let dbin = String::from_utf8(Vec::from(dbin))?;
        if dbin != "dbin" {
            return Err("Invalid dbin file".into());
        }

        let mut version: [u8; 1] = [0];
        file.read_exact(&mut version)?;

        let mut content_type: [u8; 3] = [0; 3];
        file.read_exact(&mut content_type)?;

        let content_type = String::from_utf8(Vec::from(content_type))?;

        let mut content_version: [u8; 2] = [0; 2];
        file.read_exact(&mut content_version)?;

        let content_version = String::from_utf8(Vec::from(content_version))?;

        let mut messages: Vec<Vec<u8>> = vec![];

        loop {
            match Self::read_message(&mut file) {
                Ok(message) => messages.push(message),
                Err(err) => {
                    return if err.kind() == std::io::ErrorKind::UnexpectedEof {
                        Ok(DbinFile {
                            version: version[0],
                            content_type: content_type.clone(),
                            content_version: content_version.clone(),
                            messages,
                        })
                    } else {
                        Err(err.into())
                    }
                }
            }
        }
    }

    fn read_message(file: &mut File) -> Result<Vec<u8>, std::io::Error> {
        let mut size: [u8; 4] = [0; 4];
        file.read_exact(&mut size)?;

        let size = u32::from_be_bytes(size);

        let mut content: Vec<u8> = vec![0; size as usize];
        file.read_exact(&mut content)?;

        Ok(content)
    }
}