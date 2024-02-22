use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{metadata, write, OpenOptions};
use std::io::Read;
use std::path::Path;

use crate::errors::{EraValidateError, SyncError};

#[derive(Serialize, Deserialize)]
pub struct LockEntry {
    epoch: String,
    root: String,
}

impl LockEntry {
    pub fn new(epoch: &usize, root: [u8; 32]) -> Self {
        LockEntry {
            epoch: epoch.to_string(),
            root: BASE64_STANDARD.encode(root),
        }
    }

    pub fn hash(&self) -> String {
        self.root.clone()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Lock {
    entries: HashMap<String, String>,
}

impl Lock {
    // Convenience method for creating a new Lock instance
    pub fn new() -> Self {
        Lock {
            entries: HashMap::new(),
        }
    }
}

pub fn store_last_state(file_path: &Path, entry: LockEntry) -> Result<(), Box<dyn Error>> {
    let mut lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)?;
    let mut contents = String::new();

    lock.read_to_string(&mut contents)?;
    // Attempt to deserialize the contents of the file into a `Lock` struct
    let mut json_lock: Lock = match serde_json::from_str(&contents) {
        Ok(lock) => lock,
        Err(_) => Lock::new(),
    };

    json_lock.entries.insert(entry.epoch, entry.root);

    let json_string = serde_json::to_string_pretty(&json_lock)?;
    write(file_path, json_string)?;

    Ok(())
}

pub fn check_sync_state(
    file_path: &Path,
    epoch: String,
    macc_hash: [u8; 32],
) -> Result<bool, Box<dyn Error>> {
    let file_exists = metadata(file_path).is_ok();
    if !file_exists {
        log::info!("The lockfile did not exist and was created");
    }

    let mut lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)?;
    let mut contents = String::new();

    lock.read_to_string(&mut contents)?;
    let json_lock: Lock = match serde_json::from_str(&contents) {
        Ok(lock) => lock,
        Err(_) => Lock::new(),
    };

    if !json_lock.entries.contains_key(&epoch) {
        return Ok(false);
    }

    let stored_hash = json_lock
        .entries
        .get(&epoch)
        .ok_or(SyncError::LockfileReadError)?;

    let stored_hash = BASE64_STANDARD
        .decode(&stored_hash)
        .expect("Failed to decode Base64");

    // this ensures the decoded bytes fit into a `[u8; 32]` array, which is the hash type
    let stored_hash: [u8; 32] = match stored_hash.try_into() {
        Ok(b) => b,
        Err(_) => panic!("Decoded hash does not fit into a 32-byte array"),
    };

    if macc_hash != stored_hash {
        log::error!(
            "the valid hash is: {:?} and the provided hash was: {:?}",
            macc_hash,
            stored_hash
        );
        return Err(Box::new(EraValidateError::EraAccumulatorMismatch));
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use trin_validation::accumulator::MasterAccumulator;

    #[test]
    fn test_store_last_state() -> Result<(), Box<dyn Error>> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test_lock.json");

        let entry = LockEntry {
            epoch: "0".into(),
            root: "XsH/uMOxRvQmBsdM7Zc9wW7FoQfANFhYw0P8lHgLQhg=".into(),
        };

        store_last_state(&file_path, entry)?;

        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let lock: Lock = serde_json::from_str(&contents)?;

        // test if the entry was correctly added
        assert!(lock.entries.contains_key("0"));
        assert_eq!(
            lock.entries.get("0"),
            Some(&"XsH/uMOxRvQmBsdM7Zc9wW7FoQfANFhYw0P8lHgLQhg=".into())
        );

        Ok(())
    }

    #[test]
    fn test_check_sync_state() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let file_path = dir.path().join("lockfile.json");

        let mock_json = r#"{
            "entries": {
              "0": "XsH/uMOxRvQmBsdM7Zc9wW7FoQfANFhYw0P8lHgLQhg="
            }
          }"#;

        let mut file = File::create(&file_path)?;
        writeln!(file, "{}", mock_json)?;

        let mac_file: MasterAccumulator = MasterAccumulator::default();

        // Test case where epoch exists and hashes match
        let epoch = "0".to_string();
        assert!(check_sync_state(
            &file_path,
            epoch,
            mac_file.historical_epochs[0].0
        )?);

        // Test case where epoch does not exist
        let epoch = "1".to_string();
        assert!(!check_sync_state(
            &file_path,
            epoch.clone(),
            mac_file.historical_epochs[0].0
        )?);

        // test when hashes differ but lock is present
        let epoch = "0".to_string();
        let result = check_sync_state(&file_path, epoch.clone(), mac_file.historical_epochs[1].0)
            .map_err(|error| error.to_string());

        assert_eq!(
            result.unwrap_err(),
            EraValidateError::EraAccumulatorMismatch.to_string()
        );

        Ok(())
    }
}
