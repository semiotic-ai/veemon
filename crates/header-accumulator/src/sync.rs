use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{write, File};
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct LockEntry {
    epoch: String,
    root: String,
}

impl LockEntry {
    pub fn new(epoch: &str, root: &str) -> Self {
        LockEntry {
            epoch: epoch.to_string(),
            root: root.to_string(),
        }
    }

    pub fn hash(&self) -> String {
        self.root.clone()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Lock {
    // Changed from Vec<LockEntry> to HashMap<String, String>
    entries: HashMap<String, String>,
}

pub fn store_last_state(file_path: &Path, entry: LockEntry) -> Result<(), Box<dyn Error>> {
    let mut lock = match File::open(file_path) {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            serde_json::from_str(&contents)?
        }
        Err(_) => Lock {
            entries: HashMap::new(),
        }, // File doesn't exist or couldn't be opened, start with a new Lock
    };

    lock.entries.insert(entry.epoch, entry.root);

    let json_string = serde_json::to_string_pretty(&lock)?;
    write(file_path, json_string)?;

    Ok(())
}

//TODO: function to check sync state and continue from not synced epoch
