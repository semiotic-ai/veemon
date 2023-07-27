use std::error::Error;
use std::fs::File;
use std::io::Read;

pub(crate) struct DbinFile {
    pub version: u8,
    pub content_type: String,
    pub content_version: String,
    pub message_size: u32,
    pub message: Vec<u8>,
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


        let mut message_size: [u8; 4] = [0; 4];
        file.read_exact(&mut message_size).expect("Failed to read file");

        let message_size = u32::from_be_bytes(message_size);
        println!("Message size: {}", message_size);

        let mut message: Vec<u8> = vec![0; message_size as usize];
        file.read_exact(&mut message).expect("Failed to read file");

        Ok(DbinFile {
            version: version[0],
            content_type,
            content_version,
            message_size,
            message,
        })
    }
}