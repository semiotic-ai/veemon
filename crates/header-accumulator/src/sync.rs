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
        },
    };

    lock.entries.insert(entry.epoch, entry.root);

    let json_string = serde_json::to_string_pretty(&lock)?;
    write(file_path, json_string)?;

    Ok(())
}

pub fn check_sync_state(
    file_path: &Path,
    epoch: String,
    _hash: [u8; 32],
) -> Result<bool, Box<dyn Error>> {
    let mut lock = File::open(file_path)?;

    let mut contents = String::new();

    let _ = lock.read_to_string(&mut contents)?;
    let json_lock: Lock = serde_json::from_str(&contents)?;

    if !json_lock.entries.contains_key(&epoch) {
        return Ok(false);
    }

    //TODO: validate hash, check if root sent from master_header_acc is the same as the stored in json

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

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

        // Check if the entry was correctly added
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

        // Create and write mock JSON to the temp file
        let mut file = File::create(&file_path)?;
        writeln!(file, "{}", mock_json)?;

        // Test case where epoch exists
        let epoch = "0".to_string();
        let hash: [u8; 32] = [0; 32]; // Mock hash, adjust as necessary
        assert!(check_sync_state(&file_path, epoch, hash)?);

        // Test case where epoch does not exist
        let epoch = "1".to_string();
        assert!(!check_sync_state(&file_path, epoch, hash)?);

        //TODO: test when hashes are different

        // TODO: test when are equal

        Ok(())
    }
}
